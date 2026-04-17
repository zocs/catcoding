// useAgentRoles.ts — Locale-aware role name/desc, shared across views.
//
// The role metadata (emoji + mode) lives here; the localized name and
// description are pulled from i18n `roles.<key>.{name,desc}` so they flip
// when the user switches language. Call inside a Vue setup(): `const
// { getRole, allRoles } = useAgentRoles()`.

import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

export type RoleMode = 'resident' | 'on_demand' | 'decorative'

interface RoleMeta {
  emoji: string
  mode: RoleMode
}

export const ROLE_META: Record<string, RoleMeta> = {
  pm:         { emoji: '🐱', mode: 'resident'   },
  core_dev:   { emoji: '🐱', mode: 'on_demand'  },
  frontend:   { emoji: '🐱', mode: 'on_demand'  },
  backend:    { emoji: '🐱', mode: 'on_demand'  },
  reviewer:   { emoji: '🐱', mode: 'resident'   },
  tester:     { emoji: '🐱', mode: 'on_demand'  },
  deploy:     { emoji: '🐱', mode: 'on_demand'  },
  watchdog:   { emoji: '🦉', mode: 'resident'   },
  tech_scout: { emoji: '🦊', mode: 'resident'   },
  mascot:     { emoji: '🐼', mode: 'decorative' },
}

export interface LocalizedRole {
  role: string
  emoji: string
  name: string
  desc: string
  mode: RoleMode
}

export function useAgentRoles() {
  const { t } = useI18n()

  function getRole(role: string): LocalizedRole {
    const meta = ROLE_META[role] || { emoji: '🐱', mode: 'on_demand' as RoleMode }
    return {
      role,
      emoji: meta.emoji,
      mode: meta.mode,
      name: t(`roles.${role}.name`, role),
      desc: t(`roles.${role}.desc`, ''),
    }
  }

  const allRoles = computed<Record<string, LocalizedRole>>(() => {
    const out: Record<string, LocalizedRole> = {}
    for (const key of Object.keys(ROLE_META)) {
      out[key] = getRole(key)
    }
    return out
  })

  return { getRole, allRoles, ROLE_META }
}
