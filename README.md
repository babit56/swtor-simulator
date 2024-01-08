# SWTOR Simulator

Potential folder structure
``` bash
parsed
  nodes
  objs
    apc
    dis
    abl
```

## Assumptions and other choices
- Alacrity works perfectly (>2054 makes all gcds 1.4)
- Damage ranges are sampled uniformly
- Ability cooldowns are debuffs
- Can't decide between abilities being ECS entities or normal structs. Leaning towards entities because I don't want 50% None fields

## TODO
### Soon
- [ ] Ability effects
- [ ] Talents
- [ ] Buffs
- [ ] Scenes for saving abilities
- [ ] Figure out how ability usage will be done. Using event system sounds nice.
  - [ ] Cooldowns
  - [ ] Ordering (double ticks needed)
- [ ] Decide on some (parsed) ability format. XML/Json, or just do whatever format is actually used by the game
- [ ] Should all classes (and mobs) have lists of abilities they use?

### Later
- [ ] Ask jedipedia/mari/whoever for swtor_main_global_1.tor parsing tips
- [ ] Find some list of standard buffs for players/bosses/mobs. Easy if everyone has list of their abilities and passives
- [ ] Check if weird alac is needed for dot double ticks (like pyro burning ticks)
- [ ] Rotations
  - [ ] Prio fillers
  - [ ] Priority
  - [ ] Static
  
### Done
- [x] Ability itself

## Building and running

```bash
cargo run
```
