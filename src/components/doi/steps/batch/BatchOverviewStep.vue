<template>
  <div class="flex flex-col gap-4 px-8 pb-4">
    <h2 class="text-lg font-semibold text-gray-900">{{ t('batchDoi.step3.title') }}</h2>

    <!-- Fetch progress -->
    <div v-if="fetchingAll" class="flex flex-col gap-2">
      <p class="text-sm text-gray-600">{{ t('batchDoi.step3.fetching') }}</p>
      <ProgressBar :value="fetchProgress" />
    </div>

    <!-- Summary -->
    <div v-if="!fetchingAll" class="flex gap-3 flex-wrap">
      <Tag :value="`${counts.ready} ${t('batchDoi.step3.statuses.ready')}`" severity="success" />
      <Tag
        v-if="counts.hasDoi > 0"
        :value="`${counts.hasDoi} ${t('batchDoi.step3.statuses.has_doi')}`"
        severity="warn"
      />
      <Tag
        v-if="counts.notFound > 0"
        :value="`${counts.notFound} ${t('batchDoi.step3.statuses.not_found')}`"
        severity="secondary"
      />
      <Tag
        v-if="counts.fetchError > 0"
        :value="`${counts.fetchError} ${t('batchDoi.step3.statuses.fetch_error')}`"
        severity="danger"
      />
    </div>

    <!-- Documents table -->
    <DataTable
      :value="localDocs"
      :row-class="() => 'text-sm'"
      size="small"
      scrollable
      scroll-height="400px"
    >
      <Column style="width: 3rem">
        <template #header>
          <Checkbox
            :model-value="allReadySelected"
            :indeterminate="someReadySelected && !allReadySelected"
            binary
            @change="toggleSelectAll"
          />
        </template>
        <template #body="{ data }: { data: BatchDocument }">
          <Checkbox
            v-model="data.selected"
            binary
            :disabled="data.status !== 'ready'"
          />
        </template>
      </Column>

      <Column field="halId" :header="t('batchDoi.step3.columns.halId')" style="width: 14rem">
        <template #body="{ data }: { data: BatchDocument }">
          <span class="font-mono text-xs">{{ data.halId }}</span>
        </template>
      </Column>

      <Column :header="t('batchDoi.step3.columns.title')">
        <template #body="{ data }: { data: BatchDocument }">
          <span v-if="data.status === 'loading'" class="text-gray-400 italic text-xs">
            <i class="pi pi-spin pi-spinner mr-1" />{{ t('common.loading') }}
          </span>
          <span v-else-if="data.halMetadata" class="text-gray-700">
            {{ truncate(data.halMetadata.title[0] ?? '', 60) }}
          </span>
          <span v-else-if="data.overviewError" class="text-red-500 text-xs">
            {{ truncate(data.overviewError, 60) }}
          </span>
          <span v-else class="text-gray-400 italic text-xs">—</span>
        </template>
      </Column>

      <Column :header="t('batchDoi.step3.columns.status')" style="width: 10rem">
        <template #body="{ data }: { data: BatchDocument }">
          <div class="flex flex-col gap-1">
            <Tag :severity="overviewStatusSeverity(data.status)" :value="t(`batchDoi.step3.statuses.${data.status}`)" />
            <span v-if="data.status === 'has_doi'" class="font-mono text-xs text-gray-500">
              {{ data.halMetadata?.existing_doi }}
            </span>
          </div>
        </template>
      </Column>

      <Column style="width: 5rem">
        <template #body="{ data }: { data: BatchDocument }">
          <Button
            v-if="data.status === 'fetch_error'"
            icon="pi pi-refresh"
            severity="secondary"
            text
            size="small"
            :title="t('common.retry')"
            @click="retryDoc(data)"
          />
        </template>
      </Column>
    </DataTable>

    <Message v-if="!fetchingAll && counts.ready === 0" severity="warn" :closable="false">
      {{ t('batchDoi.step3.noReadyDocs') }}
    </Message>

    <div class="flex justify-between pt-2">
      <Button
        :label="t('common.back')"
        icon="pi pi-arrow-left"
        severity="secondary"
        outlined
        :disabled="fetchingAll"
        @click="emit('back')"
      />
      <Button
        :label="t('batchDoi.step3.startProcessing')"
        icon="pi pi-play"
        icon-pos="right"
        :disabled="fetchingAll || counts.selectedReady === 0"
        @click="proceed"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import Button from 'primevue/button'
import Checkbox from 'primevue/checkbox'
import Column from 'primevue/column'
import DataTable from 'primevue/datatable'
import Message from 'primevue/message'
import ProgressBar from 'primevue/progressbar'
import Tag from 'primevue/tag'
import type { BatchDocument, BatchDocumentStatus } from '../../../../types/batch'
import type { HalMetadata } from '../SelectDocumentStep.vue'
import { truncate } from '../../../../utils/string'

const props = defineProps<{
  env: 'prod' | 'test'
  prefix: string
  documents: BatchDocument[]
}>()

const emit = defineEmits<{
  next: [documents: BatchDocument[]]
  back: []
}>()

const { t } = useI18n()

const localDocs = ref<BatchDocument[]>(props.documents.map((d) => ({ ...d })))
const fetchingAll = ref(false)
const fetchProgress = ref(0)

const counts = computed(() => {
  let ready = 0, hasDoi = 0, notFound = 0, fetchError = 0, selectedReady = 0
  for (const d of localDocs.value) {
    if (d.status === 'ready') { ready++; if (d.selected) selectedReady++ }
    else if (d.status === 'has_doi') hasDoi++
    else if (d.status === 'not_found') notFound++
    else if (d.status === 'fetch_error') fetchError++
  }
  return { ready, hasDoi, notFound, fetchError, selectedReady }
})

const allReadySelected = computed(
  () => counts.value.ready > 0 && counts.value.selectedReady === counts.value.ready,
)
const someReadySelected = computed(() => counts.value.selectedReady > 0)

function toggleSelectAll() {
  const selectAll = !allReadySelected.value
  localDocs.value.forEach((d) => {
    if (d.status === 'ready') d.selected = selectAll
  })
}

function overviewStatusSeverity(status: BatchDocumentStatus) {
  switch (status) {
    case 'ready':
      return 'success'
    case 'has_doi':
      return 'warn'
    case 'fetch_error':
      return 'danger'
    default:
      return 'secondary'
  }
}

async function fetchDoc(doc: BatchDocument) {
  doc.status = 'loading'
  doc.halMetadata = null
  doc.overviewError = null
  try {
    const meta = await invoke<HalMetadata>('get_hal_metadata', {
      halId: doc.halId,
      env: props.env,
    })
    doc.halMetadata = meta
    if (meta.existing_doi) {
      doc.status = 'has_doi'
      doc.selected = false
    } else {
      doc.status = 'ready'
    }
  } catch (err) {
    const msg = String(err)
    doc.status = msg.toLowerCase().includes('not found') ? 'not_found' : 'fetch_error'
    doc.overviewError = msg
    doc.selected = false
  }
}

async function fetchAllMetadata() {
  fetchingAll.value = true
  const total = localDocs.value.length
  for (let i = 0; i < total; i++) {
    await fetchDoc(localDocs.value[i])
    fetchProgress.value = Math.round(((i + 1) / total) * 100)
  }
  fetchingAll.value = false
}

async function retryDoc(doc: BatchDocument) {
  if (doc.status === 'loading') return
  await fetchDoc(doc)
}

function proceed() {
  emit('next', localDocs.value)
}

onMounted(() => {
  fetchAllMetadata()
})
</script>
