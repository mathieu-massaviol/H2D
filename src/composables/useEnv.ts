import { ref, watch } from 'vue'

const STORAGE_KEY = 'hal2datacite_env'

function readStored(): 'prod' | 'test' {
  const stored = localStorage.getItem(STORAGE_KEY)
  return stored === 'test' ? 'test' : 'prod'
}

// Module-level singleton — shared across all components
const env = ref<'prod' | 'test'>(readStored())

watch(env, (value) => {
  localStorage.setItem(STORAGE_KEY, value)
})

export function useEnv() {
  return { env }
}
