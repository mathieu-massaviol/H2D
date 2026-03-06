<template>
  <div class="min-h-screen bg-gray-50 py-8">
    <div class="max-w-2xl mx-auto px-6 space-y-6">
      <!-- Header -->
      <div class="flex items-center justify-between">
        <h1 class="text-2xl font-bold text-gray-900">{{ t('credentials.title') }}</h1>

        <!-- Environment selector -->
        <div class="flex items-center gap-3">
          <span class="text-sm font-medium text-gray-600">{{ t('credentials.environment') }}</span>
          <SelectButton v-model="env" :options="envOptions" option-label="label" option-value="value" />
        </div>
      </div>

      <!-- HAL section -->
      <CredentialSection
        ref="halSectionRef"
        v-bind="halSectionProps"
        :display-value="halInfo?.login ?? null"
        :saving="halSaving"
        :deleting="halDeleting"
        @save="saveHal"
        @delete="confirmDelete('hal')"
      />

      <!-- DataCite section -->
      <CredentialSection
        ref="dataciteSectionRef"
        v-bind="dataciteSectionProps"
        :display-value="dataciteInfo?.repository_id ?? null"
        :saving="dataciteSaving"
        :deleting="dataciteDeleting"
        @save="saveDatacite"
        @delete="confirmDelete('datacite')"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { useToast } from 'primevue/usetoast'
import { useConfirm } from 'primevue/useconfirm'
import { invoke } from '@tauri-apps/api/core'
import SelectButton from 'primevue/selectbutton'
import CredentialSection from '../components/credentials/CredentialSection.vue'

const { t } = useI18n()
const toast = useToast()
const confirm = useConfirm()

const env = ref<'prod' | 'test'>('prod')
const envOptions = computed(() => [
  { label: t('credentials.prod'), value: 'prod' },
  { label: t('credentials.test'), value: 'test' },
])

const halInfo = ref<{ login: string } | null>(null)
const dataciteInfo = ref<{ repository_id: string } | null>(null)
const halSaving = ref(false)
const halDeleting = ref(false)
const dataciteSaving = ref(false)
const dataciteDeleting = ref(false)

const halSectionRef = ref<InstanceType<typeof CredentialSection>>()
const dataciteSectionRef = ref<InstanceType<typeof CredentialSection>>()

// Shared action labels (same for both sections)
const sharedLabels = computed(() => ({
  configuredText: t('credentials.status.configured'),
  notConfiguredText: t('credentials.status.notConfigured'),
  editLabel: t('credentials.actions.edit'),
  deleteLabel: t('credentials.actions.delete'),
  cancelLabel: t('credentials.actions.cancel'),
  saveLabel: t('credentials.actions.save'),
}))

const halSectionProps = computed(() => ({
  ...sharedLabels.value,
  title: t('credentials.hal.title'),
  identifierLabel: t('credentials.hal.login'),
  passwordLabel: t('credentials.hal.password'),
}))

const dataciteSectionProps = computed(() => ({
  ...sharedLabels.value,
  title: t('credentials.datacite.title'),
  identifierLabel: t('credentials.datacite.repositoryId'),
  passwordLabel: t('credentials.datacite.password'),
}))

async function load(api: 'hal' | 'datacite') {
  if (api === 'hal') {
    halInfo.value = (await invoke<{ login: string } | null>('get_hal_credentials', { env: env.value })) ?? null
  } else {
    dataciteInfo.value =
      (await invoke<{ repository_id: string } | null>('get_datacite_credentials', { env: env.value })) ?? null
  }
}

async function saveHal(identifier: string, password: string) {
  halSaving.value = true
  await nextTick()
  try {
    await invoke('save_hal_credentials', { env: env.value, login: identifier, password })
    toast.add({ severity: 'success', summary: t('credentials.messages.saveSuccess'), life: 3000 })
    await load('hal')
    halSectionRef.value?.resetEditing()
  } catch (err) {
    toast.add({ severity: 'error', summary: t('credentials.messages.saveError', { error: String(err) }), life: 5000 })
  } finally {
    halSaving.value = false
  }
}

async function saveDatacite(identifier: string, password: string) {
  dataciteSaving.value = true
  await nextTick()
  try {
    await invoke('save_datacite_credentials', { env: env.value, repositoryId: identifier, password })
    toast.add({ severity: 'success', summary: t('credentials.messages.saveSuccess'), life: 3000 })
    await load('datacite')
    dataciteSectionRef.value?.resetEditing()
  } catch (err) {
    toast.add({ severity: 'error', summary: t('credentials.messages.saveError', { error: String(err) }), life: 5000 })
  } finally {
    dataciteSaving.value = false
  }
}

function confirmDelete(api: 'hal' | 'datacite') {
  const deletingRef = api === 'hal' ? halDeleting : dataciteDeleting
  confirm.require({
    message: t('credentials.messages.deleteConfirm', { api: api === 'hal' ? 'HAL' : 'DataCite' }),
    header: t('credentials.actions.delete'),
    icon: 'pi pi-exclamation-triangle',
    rejectProps: { label: t('credentials.actions.cancel'), severity: 'secondary', outlined: true },
    acceptProps: { label: t('credentials.actions.delete'), severity: 'danger' },
    accept: async () => {
      deletingRef.value = true
      try {
        await invoke('delete_credentials', { api, env: env.value })
        toast.add({ severity: 'success', summary: t('credentials.messages.deleteSuccess'), life: 3000 })
        await load(api)
      } catch (err) {
        toast.add({
          severity: 'error',
          summary: t('credentials.messages.deleteError', { error: String(err) }),
          life: 5000,
        })
      } finally {
        deletingRef.value = false
      }
    },
  })
}

watch(env, () => Promise.all([load('hal'), load('datacite')]), { immediate: true })
</script>
