<template>
  <div class="flex flex-col gap-4 max-w-xl px-8 pb-4">
    <h2 class="text-lg font-semibold text-gray-900">{{ t('createDoi.step1.title') }}</h2>

    <!-- Environment -->
    <div class="flex flex-col gap-1">
      <label class="text-sm font-medium text-gray-700">{{ t('createDoi.step1.environment') }}</label>
      <SelectButton
        v-model="localEnv"
        :options="envOptions"
        optionLabel="label"
        optionValue="value"
        :allowEmpty="false"
      />
    </div>

    <!-- DOI Prefix -->
    <div class="flex flex-col gap-1">
      <label class="text-sm font-medium text-gray-700">{{ t('createDoi.step1.prefix') }}</label>
      <Select
        v-model="localPrefix"
        :options="prefixes"
        :placeholder="loadingPrefixes ? t('createDoi.step1.loadPrefixes') : t('createDoi.step1.prefixPlaceholder')"
        :loading="loadingPrefixes"
        :disabled="loadingPrefixes || prefixes.length === 0"
        :invalid="!!prefixError"
        class="w-full"
      />
      <small v-if="prefixError" class="text-red-500">{{ prefixError }}</small>
      <small v-if="!loadingPrefixes && prefixes.length === 0" class="text-gray-400">
        {{ t('createDoi.step1.noPrefixes') }}
      </small>
    </div>

    <div class="flex justify-end pt-2">
      <Button
        :label="t('common.next')"
        icon="pi pi-arrow-right"
        iconPos="right"
        :disabled="loadingPrefixes || prefixes.length === 0"
        @click="submit"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { useToast } from 'primevue/usetoast'
import Button from 'primevue/button'
import SelectButton from 'primevue/selectbutton'
import Select from 'primevue/select'

const props = defineProps<{
  env: 'prod' | 'test'
  prefix: string
}>()

const emit = defineEmits<{
  next: [env: 'prod' | 'test', prefix: string]
}>()

const { t } = useI18n()
const toast = useToast()

const localEnv = ref<'prod' | 'test'>(props.env)
const localPrefix = ref(props.prefix || '')
const loadingPrefixes = ref(false)
const prefixes = ref<string[]>([])
const prefixError = ref('')

const envOptions = [
  { label: t('createDoi.step1.test'), value: 'test' },
  { label: t('createDoi.step1.prod'), value: 'prod' },
]

async function loadPrefixes(env: 'prod' | 'test') {
  loadingPrefixes.value = true
  prefixes.value = []
  localPrefix.value = ''
  try {
    prefixes.value = await invoke<string[]>('get_datacite_prefixes', { env })
    if (prefixes.value.length === 1) {
      localPrefix.value = prefixes.value[0]
    }
  } catch (err) {
    toast.add({
      severity: 'warn',
      summary: t('createDoi.step1.errors.prefixesFailed', { error: String(err) }),
      life: 5000,
    })
  } finally {
    loadingPrefixes.value = false
  }
}

watch(localEnv, (env) => loadPrefixes(env), { immediate: true })

function submit() {
  prefixError.value = ''
  if (!localPrefix.value) {
    prefixError.value = t('createDoi.step1.errors.prefixRequired')
    return
  }
  emit('next', localEnv.value, localPrefix.value)
}
</script>
