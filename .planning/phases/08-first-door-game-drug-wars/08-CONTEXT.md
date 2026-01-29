# Phase 8: First Door Game (Drug Wars) - Context

**Gathered:** 2026-01-28
**Status:** Ready for planning

<domain>
## Phase Boundary

**Grand Theft Meth** — a commodity trading door game where users buy/sell drugs across cities and boroughs, encounter random events, build reputation with gangs, and compete for high scores over a 90-day in-game period. Includes weapons, health, notoriety, quests, and an overarching story. This validates the door game architecture for future games.

</domain>

<decisions>
## Implementation Decisions

### Game Economy
- Starting resources: $2,000 cash, $5,500 debt to loan shark
- Loan shark: 10% daily compound interest, can borrow more anytime
- Loan shark enforcers appear as random events after hitting debt thresholds
- Bank requires $50,000 minimum to unlock; before that, only mattress stash (no interest)
- Bank earns 5% daily interest once unlocked
- Coat capacity: 4 tiers (100, 125, 150, 250 units)
- Trenchcoat guy appears randomly, offers coat upgrade but you lose inventory (and possibly weapon or money)
- Commodities: Classic drugs (Cocaine, Heroin, Acid, Weed, Meth, Speed, Ludes) plus modern ones (Fentanyl, Krokodil, Bath Salts, Tide Pods, etc.)
- Gun shop with two weapon slots: melee + gun
  - Melee: Brass knuckles, Knife, Lead pipe, Machete
  - Guns: Glock, 6 Shooter, Desert Eagle, Hunting Rifle, Shotgun, Uzi, AK-47, M16, Gatling Gun
- Casino available in some cities: Blackjack, Roulette, Horse racing

### Health & Combat
- Start at 100 HP, hospital healing costs money
- Some locations have mob doctors (required when notoriety is high)
- Notoriety/heat system: high heat increases police encounter chance
- Heat decays 10% per day if laying low
- Some drugs boost max HP but cause health drain over time (addiction mechanic)
- Claude designs specific drug effects (buffs/debuffs balanced)

### Locations & Travel
- 5-6 major cities (NYC, international like London/Tokyo)
- 3-4 boroughs per city (~20 total locations)
- Inter-city travel costs money AND time (bus cheap/slow, plane expensive/fast)
- Intra-city (borough to borough): small taxi fare, instant
- Regional economies: some cities don't have certain drugs, must travel to get them

### Gangs & Reputation
- 3 major gangs controlling territories
- Alliance through favors/quests or tribute (depends on gang)
- Risk combat in gang territory if not allied

### Quests
- Simple delivery quests with narration
- One overarching 10-15 step story quest
  - Starts like boring quest but has a hook
  - Takes player across the world
  - Ends as global kingpin (success) or dead in gutter (failure)

### Random Events
- ~15% chance per travel (occasional, not hectic)
- Positive events: Classic price deals ("Prices bottomed out!" / "Premium buyer!")
- Police encounters: Run, Fight, Talk, or Bribe
  - Talk = right dialogue, or rat someone out
  - Bribe = larger amount = higher success chance
  - Stats and weapons affect outcomes
- Muggings: Risk losing cash AND inventory, damage even if you fight or comply
- Trenchcoat guy: Random event for coat upgrades
- Loan shark enforcers: Random after debt thresholds

### Game Flow
- 90 days (extended from classic 30 due to expanded scope)
- 5 actions per day base
- Speed and similar drugs grant extra actions
- Day ends when actions depleted
- Auto-save always, resume anytime
- One save slot per user (can clear and restart)
- If story incomplete at day 90: leaderboard based on net worth score

### UI & Presentation
- BBS door game style: ANSI art headers, colored borders, atmospheric
- Global top 10 leaderboard (all-time)
- Brief intro story: Fall from grace, former dealer starting over

### Claude's Discretion
- Specific drug effects (HP boost with addiction, extra actions, etc.)
- Gang count and territory distribution
- Exact travel costs and times between cities
- Price ranges and volatility per commodity
- Combat formulas and stat effects
- Quest text and story details

</decisions>

<specifics>
## Specific Ideas

- Trenchcoat guy is the ONLY way to get the top-tier coat
- You lose everything in pockets when upgrading coat (risk/reward)
- May need to throw in weapon or large sum of money if pockets empty
- Notoriety affects which doctors you can visit (mob doctor when too hot)
- Casino as money sink/jackpot opportunity
- Story quest should feel like a boring quest at first, then reveal it's something bigger

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 08-first-door-game-drug-wars*
*Context gathered: 2026-01-28*
