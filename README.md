<div align="center">

# Tiny War
### A Rust-powered auto-battler game with solo and multiplayer modes

<br><br>
[![Play](https://gist.githubusercontent.com/cxmeel/0dbc95191f239b631c3874f4ccf114e2/raw/play.svg)](https://tvdboom.itch.io/tinywar)
<br><br>
</div>


<br>

## 📜 Overview

TinyWar is a fast-paced, real-time, auto-battler game, where players fight each other 
on a small map, with the sole goal of destroying the enemy base. A never-ending horde 
of units spawn from each base and walk down one of the three paths (top-mid-bot).
Players can queue up units to spawn, decide which path(s) the units should take, and
select from a variety of boosts to help their units win the battle. Remember! If your
base is destroyed, you lose the game.

### Combat

Units automatically attack enemy units that are in range. A unit can only attack 
one other unit at the same time, and won't change targets until the enemy has died 
or walked out of range. The damage dealt on the enemy is applied after the end of 
the attack animation (every `attack speed` seconds). Note that this means that some 
units apply damage more frequently than others, as the `attack speed` differs per unit.

Every unit has the following combat stats:

- **Health:** How much damage the unit can take before dying.
- **Physical Damage:** Base physical damage dealt on hit.
- **Magic Damage:** Base magical damage dealt on hit.
- **Armor:** Reduces incoming physical damage.
- **Magic Resist:** Reduces incoming magical damage.
- **Armor Penetration:** Reduces the target’s effective armor.
- **Magic Penetration:** Reduces the target’s effective magic resistance.

The damage calculation happens as follows:

1. Calculate defense stats of the defender:  
   `Defender_Armor = max(0, Defender::Armor - Attacker::Armor_Penetration)`  
   `Defender_Magic_Resist = max(0, Defender.Magic_Resist - Attacker.Magic_Penetration)`

2. Calculate damage mitigation using the effective defense stats:  
   `Physical_Damage = Attacker::Physical_Damage * (100 / (100 + Defender_Armor))`  
   `Magic_Damage = Attacker::Magic_Damage * (100 / (100 + Defender_Magic_Resist))`

3. Calculate final damage applied on the defender (always minimum 5):  
   `Total_Damage = max(5, Physical_Damage + Magic_Damage)`

4. Lastly, subtract the damage from the defender's health:  
   `Defender::Health -= Total_Damage`

### Boosts

Boosts are power-ups that players can use during the game to enhance their units.
Every 30 seconds, a player can choose from 3 boosts. Selected boosts become available
for activation at the top of the screen. Click on a boost to activate it.

Boosts come in two flavors:

- Instant: Apply their effect immediately upon activation. They can be recognized
  by the fact that they don't have any timer indication.
- Timed: Apply their effect for a limited duration. The timer indication on the
  bottom-right of the image indicates its length.

A player can have a maximum of 4 boosts selected/activated at the same time. If a 
player already has 4 boosts when the selection phase starts, they lose the chance to
select a new one.

## Key bindings

- `escape`: Enter/exit the in-game menu.
- `w-a-s-d`: Move the map.
- `scroll`: Zoom in/out the map.
- `space`: Pause/unpause the game.
- `ctrl + left/right arrow`: Increase/decrease the game's speed (only if host).
- `Q`: Toggle the audio settings.

- Use the arrows to select which path(s) spawning units should take.
- Every unit has a key binding to add it to the queue.
