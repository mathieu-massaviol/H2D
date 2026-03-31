<template>
  <div class="flex flex-col gap-4 pb-4 px-4">
    <h2 class="text-lg font-semibold text-gray-900">{{ t('batchDoi.step4.title') }}</h2>

    <!-- Progress bar -->
    <div class="flex flex-col gap-2">
      <div class="flex items-center justify-between">
        <span class="text-sm text-gray-600">
          {{ t('batchDoi.step4.progress', { processed: processed, total: totalToProcess }) }}
        </span>
        <span class="text-sm font-medium text-gray-700">{{ progressPercent }}%</span>
      </div>
      <ProgressBar :value="progressPercent" />
    </div>

    <!-- Cancel warning -->
    <Message
      v-if="cancelRequested && !done"
      severity="warn"
      :closable="false"
    >
      {{ t('batchDoi.step4.cancelWarning') }}
    </Message>

    <!-- Documents table -->
    <DataTable
      :value="localDocs"
      :row-class="rowClass"
      size="small"
      scrollable
      scroll-height="450px"
    >
      <Column field="halId" :header="t('batchDoi.step3.columns.halId')" style="width: 14rem">
        <template #body="{ data }: { data: BatchDocument }">
          <span class="font-mono text-xs">{{ data.halId }}</span>
        </template>
      </Column>

      <Column :header="t('batchDoi.step3.columns.title')">
        <template #body="{ data }: { data: BatchDocument }">
          <span class="text-gray-700 text-sm">
            {{ truncate(data.halMetadata?.title[0] ?? data.halId, 55) }}
          </span>
        </template>
      </Column>

      <Column :header="t('batchDoi.step3.columns.status')" style="width: 10rem">
        <template #body="{ data }: { data: BatchDocument }">
          <div class="flex items-center gap-2">
            <i v-if="data.status === 'processing'" class="pi pi-spin pi-spinner text-blue-500" />
            <Tag
              v-else
              :severity="batchStatusSeverity(data.status)"
              :value="t(`batchDoi.step4.statuses.${data.status}`)"
            />
          </div>
        </template>
      </Column>

      <Column field="doi" :header="t('batchDoi.step5.columns.doi')" style="width: 14rem">
        <template #body="{ data }: { data: BatchDocument }">
          <span v-if="data.doi" class="font-mono text-xs text-blue-600">{{ data.doi }}</span>
          <span v-else class="text-gray-400 text-xs">—</span>
        </template>
      </Column>

      <Column style="width: 5rem" :header="t('batchDoi.step5.columns.halSynced')">
        <template #body="{ data }: { data: BatchDocument }">
          <i
            v-if="data.status === 'success'"
            :class="data.halSynced ? 'pi pi-check text-green-500' : 'pi pi-times text-orange-400'"
          />
        </template>
      </Column>
    </DataTable>

    <!-- Done summary -->
    <div v-if="done" class="flex flex-col gap-3">
      <div class="flex gap-3 flex-wrap">
        <Tag
          :value="`${resultCounts.success} ${t('batchDoi.step4.statuses.success')}`"
          severity="success"
        />
        <Tag
          v-if="resultCounts.error > 0"
          :value="`${resultCounts.error} ${t('batchDoi.step4.statuses.error')}`"
          severity="danger"
        />
        <Tag
          v-if="resultCounts.skipped > 0"
          :value="`${resultCounts.skipped} ${t('batchDoi.step4.skipped')}`"
          severity="secondary"
        />
      </div>

      <div class="flex justify-end pt-2">
        <Button
          :label="t('batchDoi.step4.viewResults')"
          icon="pi pi-arrow-right"
          icon-pos="right"
          @click="emit('next', localDocs)"
        />
      </div>
    </div>

    <!-- Cancel button during processing -->
    <div v-else class="flex justify-end pt-2">
      <Button
        :label="t('common.cancel')"
        icon="pi pi-stop"
        severity="secondary"
        outlined
        :disabled="cancelRequested"
        @click="cancelRequested = true"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import Button from 'primevue/button'
import Column from 'primevue/column'
import DataTable from 'primevue/datatable'
import Message from 'primevue/message'
import ProgressBar from 'primevue/progressbar'
import Tag from 'primevue/tag'
import type { BatchDocument } from '../../../../types/batch'
import { batchStatusSeverity } from '../../../../types/batch'
import { truncate } from '../../../../utils/string'

const props = defineProps<{
  env: 'prod' | 'test'
  prefix: string
  suffixPrefix: string
  documents: BatchDocument[]
}>()

const emit = defineEmits<{
  next: [documents: BatchDocument[]]
}>()

const { t } = useI18n()

const localDocs = ref<BatchDocument[]>(props.documents.map((d) => ({ ...d })))
const cancelRequested = ref(false)
const done = ref(false)
const processed = ref(0)
const totalToProcess = ref(0)
const progressPercent = computed(() =>
  totalToProcess.value === 0 ? 0 : Math.round((processed.value / totalToProcess.value) * 100),
)
const resultCounts = computed(() => {
  let success = 0, error = 0, skipped = 0
  for (const d of localDocs.value) {
    if (d.status === 'success') success++
    else if (d.status === 'error') error++
    else if (d.status === 'has_doi' || d.status === 'not_found' || d.status === 'fetch_error') skipped++
  }
  return { success, error, skipped }
})

function rowClass(data: BatchDocument) {
  return data.status === 'processing' ? 'bg-blue-50' : ''
}

async function startProcessing() {
  const queue = localDocs.value.filter((d) => d.selected && d.status === 'ready')
  totalToProcess.value = queue.length

  for (const doc of queue) {
    if (cancelRequested.value) break

    doc.status = 'processing'
    doc.doi = null
    doc.halSynced = false
    doc.processError = null

    try {
      // Step 1: create draft DOI
      doc.doi = await invoke<string>('create_doi', {
        halMetadata: doc.halMetadata!,
        prefix: props.prefix,
        env: props.env,
        publish: false,
        suffixPrefix: props.suffixPrefix,
      })

      // Step 2: publish DOI
      await invoke('publish_doi', { doi: doc.doi, env: props.env })

      // Step 3: sync to HAL (best-effort — failure doesn't fail the record)
      try {
        await invoke('add_doi_to_hal_record', {
          halId: doc.halId,
          doi: doc.doi,
          env: props.env,
        })
        doc.halSynced = true
      } catch (halErr) {
        doc.halSynced = false
        doc.processError =
          String(halErr) === 'HAL_NO_WRITE_ACCESS'
            ? t('batchDoi.step5.halNoWriteAccess')
            : String(halErr)
      }

      doc.status = 'success'
    } catch (err) {
      doc.status = 'error'
      doc.processError = String(err)
      // Best-effort cleanup of draft DOI
      if (doc.doi) {
        invoke('delete_doi', { doi: doc.doi, env: props.env }).catch(() => {})
        doc.doi = null
      }
    }

    processed.value++
  }

  done.value = true
}

onMounted(() => {
  startProcessing()
})
</script>
