# Parallel Game Development Workflow

## Overview

Use Claude Code as an orchestrator to parallelize research/planning across multiple games, consolidate all questions into a single Q&A session, then execute in parallel.

## Setup (Each Session)

### 1. Start Claude Code
```powershell
cd C:\git\bbs
claude
```

### 2. Invoke Parallel Research
Tell Claude:
```
Let's do parallel game development. Here are the games I want to work on:
- [Game 1]: [brief description]
- [Game 2]: [brief description]
- [Game 3]: [brief description]

Spawn research agents for each, consolidate questions, then I'll answer in one batch.
```

### 3. Answer Consolidated Questions
Claude will present all questions from all games in one batch. Answer them all at once.

### 4. Parallel Execution
After Q&A, Claude spawns parallel planning/execution agents. Options:
- **Same session**: Background agents in this Claude instance
- **Worktrees**: For true parallel execution across multiple games

## Workflow Phases

```
┌─────────────────────────────────────────────────────────────────┐
│ PHASE 1: Parallel Research (5-10 min)                          │
│ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐               │
│ │Research │ │Research │ │Research │ │Research │  (background) │
│ │ Game A  │ │ Game B  │ │ Game C  │ │ Game D  │               │
│ └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘               │
│      │           │           │           │                     │
│      └───────────┴─────┬─────┴───────────┘                     │
│                        ▼                                        │
│              ┌─────────────────┐                               │
│              │ Consolidate Qs  │                               │
│              └────────┬────────┘                               │
└───────────────────────┼─────────────────────────────────────────┘
                        ▼
┌─────────────────────────────────────────────────────────────────┐
│ PHASE 2: Batch Q&A (10-15 min)                                 │
│              ┌─────────────────┐                               │
│              │  YOU answer all │                               │
│              │  questions once │                               │
│              └────────┬────────┘                               │
└───────────────────────┼─────────────────────────────────────────┘
                        ▼
┌─────────────────────────────────────────────────────────────────┐
│ PHASE 3: Parallel Execution                                    │
│ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐               │
│ │ Plan &  │ │ Plan &  │ │ Plan &  │ │ Plan &  │  (parallel)   │
│ │Execute A│ │Execute B│ │Execute C│ │Execute D│               │
│ └─────────┘ └─────────┘ └─────────┘ └─────────┘               │
└─────────────────────────────────────────────────────────────────┘
```

## Tips

- **3-4 games max** per batch for manageable Q&A
- Games with similar mechanics can share answers
- If a game needs deep discussion, pull it out for dedicated planning
- Use `/gsd:discuss-phase` for complex games that need more back-and-forth

## Context Management

### When to Clear Context
- After completing a batch of games (before starting next batch)
- When context feels slow/bloated (long responses, repetition)
- After major milestones
- When switching between unrelated work streams

### Preserving State Before Reset

**Option 1: GSD Commands (Recommended)**
```
/gsd:pause-work
```
This creates a handoff document with current state, next steps, and blockers.

**Option 2: Manual Checkpoint**
Ask Claude:
```
Create a context checkpoint before I reset. Capture:
- What we were working on
- Decisions made
- Next steps
- Any open questions
```
This writes to `.planning/CHECKPOINT.md` or similar.

**Option 3: Quick State Dump**
```
Summarize current state to .planning/SESSION-STATE.md
```

### Resuming After Reset

**Option 1: GSD Resume**
```
/gsd:resume-work
```
Restores from pause-work handoff.

**Option 2: Manual Resume**
```
Read .planning/PARALLEL-WORKFLOW.md and .planning/SESSION-STATE.md, then continue where we left off.
```

**Option 3: Fresh Start with Context**
```
New session. Read CLAUDE.md for project context.
We're doing parallel game dev - here are the games: [list]
Previous session completed: [X, Y]
Continue with: [Z]
```

### What Persists Automatically
- `CLAUDE.md` - Project guidance (Claude reads this automatically)
- `.planning/PROJECT.md` - Project vision and requirements
- `.planning/ROADMAP.md` - Phase status
- `.planning/phases/*/` - All research, plans, and summaries
- Git history - All committed code

### What Needs Manual Preservation
- In-flight decisions not yet in a PLAN.md
- Answers to consolidated questions (write to phase CONTEXT.md)
- Current batch of games being worked on
- Which research agents completed vs pending

### Pro Tip: Anchor Documents
Before spawning parallel agents, tell Claude:
```
Write consolidated Q&A answers to .planning/BATCH-ANSWERS.md so we don't lose them on reset.
```

## Example Prompt

```
Parallel game dev session. Research these games and consolidate questions:

1. Trade Wars clone - space trading, sector-based travel, ports
2. Blackjack - casino card game with betting
3. Hangman - word guessing with ASCII art
4. Global Thermonuclear War - WarGames reference, simple strategy

Spawn 4 research agents, gather assumptions/questions, present as single list.
```
