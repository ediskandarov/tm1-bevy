# Imported Source

- Repository: https://github.com/ediskandarov/tm1_decomp
- Commit: `fb8f9dd4c3aff275893c2573c432a94f644e9970`
- Original path: `src/tm1/TIMER.c`
- Local file: `TIMER.c`, unchanged
- SHA-256: `4fab2b0850fae5d31a7990cb2d28332cb2e19ea12e0791b463ee9bd11667f122`
- License: MIT; original notice retained in `LICENSE.md`

Also imported unchanged from the same commit:

- Original path: `src/tm1/MATH.c`
- Local file: `MATH.c`
- SHA-256: `66d2a900369398069f679aef2cf324f7c57f2ed81c13c19dd466c6cd6de2d3a1`

The adjacent host headers and timer harness are new compatibility code. They
provide a controlled timing probe and fixed-width math layouts, not a complete
PS1 SDK or BIOS. The positive-domain square-root replacement is in Rust.

The driving files use functions from the same pinned source. `car_dynamics.c`
includes the CAR_INIT stat initializers for the ice cream truck, taxi, semi, monster truck, Lamborghini, Harley, low rider and dune buggy. Their projectile special parameters come from INTERACT.c. It also contains CAR, CAR_INIT, SMOOTH and UA pad functions. The expression
`t & (t | (s32)sk)` reduces to `t`, avoiding a host pointer truncation.
`arena_ground.c` uses BRIDGES, POTHOLES, TRIGPTS, CARUPD, CAR and SMOOTH.
The Warehouse initializers come from TRIGPTS, BRIDGES, CURBS and POTHOLES.
Its tire-height routine excludes the rooftop-only branch. `ai_driving.c` uses
AI_CAR, AICARINI, CARUPD, CAR and CAR_INIT for the regular opponents. Their route and attack profiles come from AICARINI. Weapon chances, firing conditions and front/rear selection weights come from AICARUPD. The arena adapters replace sound and presentation calls while retaining gameplay effects and random draws on compared paths. Weapon-profile
addresses in the car layout have their original 32-bit width.
The host supplies the level and player context, copies records across the ABI
and serializes C globals. The game executable and SDK tables stay on the disc.

(This text was generated via LLM.)

The arena collision traversal and point queries derive from HD.c and HDP.c.
The contact/damage, machine-gun, missile, taser, drop and pickup paths use
CARUPD.c, CAR.c, WEAPON.c, INTERACT.c, EXPLODE.c and their related arena
updates at the pinned revision. Host graph decoding resolves disc addresses
to native pointers and converts target references to stable arena IDs. C
sources with arena-specific names are adapted subsets. The untouched MATH.c
and TIMER.c hashes above identify the two byte-for-byte imports.

Hammerhead uses CAR.c InitMonsterSmash and CARUPD.c CheckMonsterSmash. The host resolves its target against the active opponent and submits the crush effects through the arena effect queue.

Front and rear flamethrowers follow INTERACT.c fireFlameThrower and displayFlamethrowers. Particle velocity, animation and lifetime follow EXPLODE.c do_flamethrower_burst and animate_explosions. GRUTILS.c grutilsParse3DSprite supplies the frame UV layout.

Warthog, Spectre and Road Kill use the Humvee, Corvette and Mad Max profiles from CAR_INIT.c. Their launch mounts and ammunition rules follow INTERACT.c. Missile guidance, the post-impact update through life -11 and the contrail ring geometry follow WEAPON.c. Trail widths and UV orientation use the original CONTRAIL_PT and CONTRAIL_UV expressions.
