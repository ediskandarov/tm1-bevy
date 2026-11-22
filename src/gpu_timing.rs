//! GPU timestamp probes around the Bevy render graph in the capture run.
//! Metal supports pass boundary queries on hardware without in-pass sampling.
use bevy::{
    prelude::*,
    render::{
        RenderApp,
        renderer::{
            PendingCommandBuffers, RenderDevice, RenderGraph, RenderGraphSystems, RenderQueue,
        },
    },
};
use wgpu::{
    BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Features, MapMode, QuerySet,
    QuerySetDescriptor, QueryType,
};
#[derive(Resource, Default)]
struct Probe {
    frame: u32,
    queries: Option<QuerySet>,
    target: Option<wgpu::TextureView>,
}
pub struct GpuTiming;
impl Plugin for GpuTiming {
    fn build(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<Probe>()
            .add_systems(
                RenderGraph,
                (
                    begin
                        .after(RenderGraphSystems::Begin)
                        .before(RenderGraphSystems::Render),
                    end.after(RenderGraphSystems::Render)
                        .before(RenderGraphSystems::Submit),
                ),
            );
    }
}
const WARMUP: u32 = 20;
const COUNT: u32 = 80;
fn mark(e: &mut wgpu::CommandEncoder, q: &QuerySet, view: &wgpu::TextureView, index: u32) {
    let attachments = [Some(wgpu::RenderPassColorAttachment {
        view,
        resolve_target: None,
        depth_slice: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            store: wgpu::StoreOp::Store,
        },
    })];
    drop(e.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("GPU frame boundary"),
        color_attachments: &attachments,
        timestamp_writes: Some(wgpu::RenderPassTimestampWrites {
            query_set: q,
            beginning_of_pass_write_index: None,
            end_of_pass_write_index: Some(index),
        }),
        ..Default::default()
    }));
}
fn begin(
    device: Res<RenderDevice>,
    mut probe: ResMut<Probe>,
    mut buffers: ResMut<PendingCommandBuffers>,
) {
    probe.frame += 1;
    if !(WARMUP + 1..=WARMUP + COUNT).contains(&probe.frame) {
        return;
    }
    if !device.features().contains(Features::TIMESTAMP_QUERY) {
        if probe.frame == WARMUP + 1 {
            eprintln!("GPU pass-boundary timestamps unavailable");
        }
        return;
    }
    let index = (probe.frame - WARMUP - 1) * 2;
    let view = probe
        .target
        .get_or_insert_with(|| {
            device
                .wgpu_device()
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("timestamp marker"),
                    size: wgpu::Extent3d {
                        width: 1,
                        height: 1,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                })
                .create_view(&Default::default())
        })
        .clone();
    let q = probe.queries.get_or_insert_with(|| {
        device.wgpu_device().create_query_set(&QuerySetDescriptor {
            label: Some("arena graph boundaries"),
            ty: QueryType::Timestamp,
            count: COUNT * 2,
        })
    });
    let mut e = device.create_command_encoder(&CommandEncoderDescriptor::default());
    mark(&mut e, q, &view, index);
    buffers.push_encoder(e, "arena timing start");
}
fn end(
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    probe: Res<Probe>,
    mut buffers: ResMut<PendingCommandBuffers>,
) {
    let Some(q) = &probe.queries else { return };
    if !(WARMUP + 1..=WARMUP + COUNT).contains(&probe.frame) {
        return;
    }
    let index = (probe.frame - WARMUP - 1) * 2 + 1;
    let mut e = device.create_command_encoder(&CommandEncoderDescriptor::default());
    mark(&mut e, q, probe.target.as_ref().unwrap(), index);
    if index == COUNT * 2 - 1 {
        let bytes = 16 * u64::from(COUNT);
        let alloc = |usage| {
            device.wgpu_device().create_buffer(&BufferDescriptor {
                label: None,
                size: bytes,
                usage,
                mapped_at_creation: false,
            })
        };
        let resolve = alloc(BufferUsages::QUERY_RESOLVE | BufferUsages::COPY_SRC);
        let read = alloc(BufferUsages::COPY_DST | BufferUsages::MAP_READ);
        e.resolve_query_set(q, 0..COUNT * 2, &resolve, 0);
        e.copy_buffer_to_buffer(&resolve, 0, &read, 0, bytes);
        let data = read.clone();
        let period = f64::from(queue.get_timestamp_period());
        e.map_buffer_on_submit(&read, MapMode::Read, .., move |result| {
            if let Err(err) = result {
                eprintln!("GPU statistics readback failed: {err}");
                return;
            }
            let mapped = data.slice(..).get_mapped_range().unwrap();
            let mut ms = Vec::new();
            for pair in mapped.chunks_exact(16) {
                let a = u64::from_le_bytes(pair[..8].try_into().unwrap());
                let b = u64::from_le_bytes(pair[8..].try_into().unwrap());
                if b > a && a != 0 {
                    ms.push((b - a) as f64 * period * 1e-6);
                }
            }
            ms.sort_by(f64::total_cmp);
            if !ms.is_empty() {
                eprintln!(
                    "GPU graph interval, {} timestamp pairs: median {:.4} ms, p95 {:.4} ms",
                    ms.len(),
                    ms[ms.len() / 2],
                    ms[(ms.len() - 1) * 95 / 100]
                );
            } else {
                eprintln!("GPU returned no valid timestamp pairs");
            }
        });
    }
    buffers.push_encoder(e, "arena timing end");
}
