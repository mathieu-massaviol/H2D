<template>
  <div class="min-h-screen bg-gray-50 py-page-y px-page-x space-y-6">
    <h1 class="text-2xl font-bold text-gray-900">{{ t('credentials.title') }}</h1>

    <div class="flex flex-col lg:flex-row gap-6">
      <!-- Production column -->
      <div class="flex-1 space-y-4">
        <h2 class="font-semibold text-gray-700 flex items-center gap-2">
          <Tag :value="t('credentials.prod')" severity="danger" />
        </h2>
        <CredentialSection
          ref="halSectionProdRef"
          v-bind="halSectionProps"
          :display-value="halInfoProd?.login ?? null"
          :saving="halSavingProd"
          :deleting="halDeletingProd"
          @save="(id, pw) => saveHal('prod', id, pw)"
          @delete="confirmDelete('hal', 'prod')"
        />
        <CredentialSection
          ref="dataciteSectionProdRef"
          v-bind="dataciteSectionProps"
          :display-value="dataciteInfoProd?.repository_id ?? null"
          :saving="dataciteSavingProd"
          :deleting="dataciteDeletingProd"
          @save="(id, pw) => saveDatacite('prod', id, pw)"
          @delete="confirmDelete('datacite', 'prod')"
        />
      </div>

      <!-- Test column -->
      <div class="flex-1 space-y-4">
        <h2 class="font-semibold text-gray-700 flex items-center gap-2">
          <Tag :value="t('credentials.test')" severity="secondary" />
        </h2>
        <CredentialSection
          ref="halSectionTestRef"
          v-bind="halSectionProps"
          :display-value="halInfoTest?.login ?? null"
          :saving="halSavingTest"
          :deleting="halDeletingTest"
          @save="(id, pw) => saveHal('test', id, pw)"
          @delete="confirmDelete('hal', 'test')"
        />
        <CredentialSection
          ref="dataciteSectionTestRef"
          v-bind="dataciteSectionProps"
          :display-value="dataciteInfoTest?.repository_id ?? null"
          :saving="dataciteSavingTest"
          :deleting="dataciteDeletingTest"
          @save="(id, pw) => saveDatacite('test', id, pw)"
          @delete="confirmDelete('datacite', 'test')"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useToast } from 'primevue/usetoast'
import { useConfirm } from 'primevue/useconfirm'
import { invoke } from '@tauri-apps/api/core'
import Tag from 'primevue/tag'
import CredentialSection from '../components/credentials/CredentialSection.vue'

const { t } = useI18n()
const toast = useToast()
const confirm = useConfirm()

// Per-env credential info
const halInfoProd = ref<{ login: string } | null>(null)
const halInfoTest = ref<{ login: string } | null>(null)
const dataciteInfoProd = ref<{ repository_id: string } | null>(null)
const dataciteInfoTest = ref<{ repository_id: string } | null>(null)

// Loading states
const halSavingProd = ref(false)
const halSavingTest = ref(false)
const halDeletingProd = ref(false)
const halDeletingTest = ref(false)
const dataciteSavingProd = ref(false)
const dataciteSavingTest = ref(false)
const dataciteDeletingProd = ref(false)
const dataciteDeletingTest = ref(false)

// Section refs
const halSectionProdRef = ref<InstanceType<typeof CredentialSection>>()
const halSectionTestRef = ref<InstanceType<typeof CredentialSection>>()
const dataciteSectionProdRef = ref<InstanceType<typeof CredentialSection>>()
const dataciteSectionTestRef = ref<InstanceType<typeof CredentialSection>>()

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

async function load(api: 'hal' | 'datacite', env: 'prod' | 'test') {
  if (api === 'hal') {
    const result = (await invoke<{ login: string } | null>('get_hal_credentials', { env })) ?? null
    env === 'prod' ? (halInfoProd.value = result) : (halInfoTest.value = result)
  } else {
    const result =
      (await invoke<{ repository_id: string } | null>('get_datacite_credentials', { env })) ?? null
    env === 'prod' ? (dataciteInfoProd.value = result) : (dataciteInfoTest.value = result)
  }
}

async function saveHal(env: 'prod' | 'test', identifier: string, password: string) {
  const savingRef = env === 'prod' ? halSavingProd : halSavingTest
  const sectionRef = env === 'prod' ? halSectionProdRef : halSectionTestRef
  savingRef.value = true
  await nextTick()
  try {
    await invoke('save_hal_credentials', { env, login: identifier, password })
    toast.add({ severity: 'success', summary: t('credentials.messages.saveSuccess'), life: 3000 })
    await load('hal', env)
    sectionRef.value?.resetEditing()
  } catch (err) {
    toast.add({ severity: 'error', summary: t('credentials.messages.saveError', { error: String(err) }), life: 5000 })
  } finally {
    savingRef.value = false
  }
}

async function saveDatacite(env: 'prod' | 'test', identifier: string, password: string) {
  const savingRef = env === 'prod' ? dataciteSavingProd : dataciteSavingTest
  const sectionRef = env === 'prod' ? dataciteSectionProdRef : dataciteSectionTestRef
  savingRef.value = true
  await nextTick()
  try {
    await invoke('save_datacite_credentials', { env, repositoryId: identifier, password })
    toast.add({ severity: 'success', summary: t('credentials.messages.saveSuccess'), life: 3000 })
    await load('datacite', env)
    sectionRef.value?.resetEditing()
  } catch (err) {
    toast.add({ severity: 'error', summary: t('credentials.messages.saveError', { error: String(err) }), life: 5000 })
  } finally {
    savingRef.value = false
  }
}

function confirmDelete(api: 'hal' | 'datacite', env: 'prod' | 'test') {
  const deletingRef =
    api === 'hal'
      ? env === 'prod' ? halDeletingProd : halDeletingTest
      : env === 'prod' ? dataciteDeletingProd : dataciteDeletingTest

  confirm.require({
    message: t('credentials.messages.deleteConfirm', { api: api === 'hal' ? 'HAL' : 'DataCite' }),
    header: t('credentials.actions.delete'),
    icon: 'pi pi-exclamation-triangle',
    rejectProps: { label: t('credentials.actions.cancel'), severity: 'secondary', outlined: true },
    acceptProps: { label: t('credentials.actions.delete'), severity: 'danger' },
    accept: async () => {
      deletingRef.value = true
      try {
        await invoke('delete_credentials', { api, env })
        toast.add({ severity: 'success', summary: t('credentials.messages.deleteSuccess'), life: 3000 })
        await load(api, env)
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

onMounted(() =>
  Promise.all([
    load('hal', 'prod'),
    load('hal', 'test'),
    load('datacite', 'prod'),
    load('datacite', 'test'),
  ]),
)
</script>
