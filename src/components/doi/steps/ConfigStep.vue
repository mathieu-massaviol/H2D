<template>
  <div class="flex flex-col gap-4 max-w-xl mx-auto pb-4 px-4">
    <h2 class="text-lg font-semibold text-gray-900">{{ t('createDoi.step1.title') }}</h2>

    <!-- Credentials missing warning -->
    <Message v-if="credentialsMissing" severity="warn" :closable="false">
      {{ t('createDoi.step1.credentialsMissing') }}
      <RouterLink to="/credentials" class="underline ml-1">{{ t('nav.credentials') }}</RouterLink>
    </Message>

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

    <!-- DOI Suffix Prefix -->
    <div class="flex flex-col gap-1">
      <label class="text-sm font-medium text-gray-700">{{ t('createDoi.step1.suffixPrefix') }}</label>
      <InputText
        v-model="localSuffixPrefix"
        :placeholder="t('createDoi.step1.suffixPrefixPlaceholder')"
        :invalid="!!suffixPrefixError"
        class="w-full"
      />
      <small v-if="suffixPrefixError" class="text-red-500">{{ suffixPrefixError }}</small>
      <small v-else class="text-gray-400">
        {{ t('createDoi.step1.suffixPrefixHint', { prefix: localPrefix || '10.XXXXX', suffix: localSuffixPrefix || 'your-prefix' }) }}
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
import InputText from 'primevue/inputtext'
import Message from 'primevue/message'
import Select from 'primevue/select'
import { RouterLink } from 'vue-router'
import { useEnv } from '../../../composables/useEnv'

const props = defineProps<{
  prefix: string
  suffixPrefix: string
}>()

const emit = defineEmits<{
  next: [prefix: string, suffixPrefix: string]
}>()

const { t } = useI18n()
const toast = useToast()
const { env } = useEnv()

const localPrefix = ref(props.prefix || '')
const localSuffixPrefix = ref(props.suffixPrefix || '')
const loadingPrefixes = ref(false)
const prefixes = ref<string[]>([])
const prefixError = ref('')
const suffixPrefixError = ref('')
const credentialsMissing = ref(false)

async function loadPrefixes(env: 'prod' | 'test') {
  loadingPrefixes.value = true
  credentialsMissing.value = false
  prefixes.value = []
  localPrefix.value = ''
  try {
    const creds = await invoke<unknown>('get_datacite_credentials', { env })
    if (!creds) {
      credentialsMissing.value = true
      return
    }
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

watch(env, (e) => loadPrefixes(e), { immediate: true })

function submit() {
  prefixError.value = ''
  suffixPrefixError.value = ''
  if (!localPrefix.value) {
    prefixError.value = t('createDoi.step1.errors.prefixRequired')
    return
  }
  if (localSuffixPrefix.value && !/^[a-z0-9-]+$/.test(localSuffixPrefix.value)) {
    suffixPrefixError.value = t('createDoi.step1.errors.suffixPrefixInvalid')
    return
  }
  emit('next', localPrefix.value, localSuffixPrefix.value)
}
</script>
