import type { AgentStatus } from './types'

export type Breed =
  | 'siamese' | 'british_blue' | 'orange_tabby' | 'maine_coon'
  | 'black' | 'abyssinian' | 'dragon_li' | 'owl' | 'fox' | 'panda' | 'generic'

export type SpriteState = 'working' | 'idle' | 'sleeping' | 'eating' | 'playing'

export const ROLE_TO_BREED: Record<string, Breed> = {
  pm: 'siamese',
  core_dev: 'british_blue',
  frontend: 'orange_tabby',
  backend: 'maine_coon',
  reviewer: 'black',
  tester: 'abyssinian',
  deploy: 'dragon_li',
  watchdog: 'owl',
  tech_scout: 'fox',
  mascot: 'panda',
}

export function breedFor(role: string): Breed {
  return ROLE_TO_BREED[role] || 'generic'
}

export function agentStatusToSpriteState(status: AgentStatus): SpriteState {
  if (status === 'active' || status === 'busy') return 'working'
  if (status === 'error') return 'idle'
  return 'idle'
}
