<template>
  <div class="flex flex-col gap-4 px-8 pb-4">
    <h2 class="text-lg font-semibold text-gray-900">{{ t('batchDoi.step5.title') }}</h2>

    <!-- Summary chips -->
    <div class="flex gap-3 flex-wrap">
      <Tag
        :value="`${counts.success} ${t('batchDoi.step4.statuses.success')}`"
        severity="success"
      />
      <Tag
        v-if="counts.error > 0"
        :value="`${counts.error} ${t('batchDoi.step4.statuses.error')}`"
        severity="danger"
      />
      <Tag
        v-if="counts.skipped > 0"
        :value="`${counts.skipped} ${t('batchDoi.step4.skipped')}`"
        severity="secondary"
      />
      <Tag
        v-if="counts.halSyncFail > 0"
        :value="`${counts.halSyncFail} ${t('batchDoi.step5.halSyncFailed')}`"
        severity="warn"
      />
    </div>

    <!-- Results table -->
    <DataTable :value="props.documents" size="small" scrollable scroll-height="400px">
      <Column field="halId" :header="t('batchDoi.step5.columns.halId')" style="width: 12rem">
        <template #body="{ data }: { data: BatchDocument }">
          <span class="font-mono text-xs">{{ data.halId }}</span>
        </template>
      </Column>

      <Column :header="t('batchDoi.step5.columns.doi')" style="width: 16rem">
        <template #body="{ data }: { data: BatchDocument }">
          <a
            v-if="data.doi"
            :href="`https://doi.org/${data.doi}`"
            target="_blank"
            rel="noopener"
            class="font-mono text-xs text-blue-600 hover:underline"
          >
            {{ data.doi }}
          </a>
          <span v-else class="text-gray-400 text-xs">—</span>
        </template>
      </Column>

      <Column :header="t('batchDoi.step5.columns.status')" style="width: 10rem">
        <template #body="{ data }: { data: BatchDocument }">
          <Tag
            :severity="batchStatusSeverity(data.status)"
            :value="t(`batchDoi.step5.statuses.${data.status}`)"
          />
        </template>
      </Column>

      <Column :header="t('batchDoi.step5.columns.halSynced')" style="width: 7rem">
        <template #body="{ data }: { data: BatchDocument }">
          <span v-if="data.status === 'success'">
            <i
              :class="
                data.halSynced
                  ? 'pi pi-check text-green-500'
                  : 'pi pi-times text-orange-400'
              "
            />
          </span>
          <span v-else class="text-gray-400 text-xs">—</span>
        </template>
      </Column>

      <Column :header="t('batchDoi.step5.columns.error')">
        <template #body="{ data }: { data: BatchDocument }">
          <span
            v-if="data.processError || data.overviewError"
            class="text-red-500 text-xs"
            :title="data.processError ?? data.overviewError ?? ''"
          >
            {{ truncate(data.processError ?? data.overviewError ?? '', 60) }}
          </span>
          <span v-else class="text-gray-400 text-xs">—</span>
        </template>
      </Column>
    </DataTable>

    <div class="flex justify-between pt-2">
      <Button
        :label="t('batchDoi.step5.exportCsv')"
        icon="pi pi-download"
        severity="secondary"
        outlined
        :loading="exporting"
        @click="exportCsv"
      />
      <Button
        :label="t('common.backToHome')"
        icon="pi pi-home"
        @click="emit('home')"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { useToast } from 'primevue/usetoast'
import Button from 'primevue/button'
import Column from 'primevue/column'
import DataTable from 'primevue/datatable'
import Tag from 'primevue/tag'
import type { BatchDocument } from '../../../../types/batch'
import { batchStatusSeverity } from '../../../../types/batch'
import { truncate } from '../../../../utils/string'

const props = defineProps<{
  documents: BatchDocument[]
  env: 'prod' | 'test'
}>()

const emit = defineEmits<{
  home: []
}>()

const { t } = useI18n()
const toast = useToast()
const exporting = ref(false)

const counts = computed(() => {
  let success = 0, error = 0, skipped = 0, halSyncFail = 0
  for (const d of props.documents) {
    if (d.status === 'success') { success++; if (!d.halSynced) halSyncFail++ }
    else if (d.status === 'error') error++
    else if (d.status === 'has_doi' || d.status === 'not_found' || d.status === 'fetch_error') skipped++
  }
  return { success, error, skipped, halSyncFail }
})

async function exportCsv() {
  const headers = [
    t('batchDoi.step5.columns.halId'),
    t('batchDoi.step5.columns.doi'),
    t('batchDoi.step5.columns.status'),
    t('batchDoi.step5.columns.halSynced'),
    t('batchDoi.step5.columns.error'),
  ]

  const rows = props.documents.map((doc) => [
    doc.halId,
    doc.doi ?? '',
    doc.status,
    doc.doi ? (doc.halSynced ? 'yes' : 'no') : '',
    doc.processError ?? doc.overviewError ?? '',
  ])

  // UTF-8 BOM for Excel compatibility
  const csvContent =
    '\uFEFF' +
    [headers, ...rows]
      .map((row) => row.map((cell) => `"${String(cell).replace(/"/g, '""')}"`).join(','))
      .join('\n')

  const defaultFilename = `batch-doi-${new Date().toISOString().slice(0, 10)}.csv`

  exporting.value = true
  try {
    const savedPath = await invoke<string | null>('save_csv', {
      content: csvContent,
      defaultFilename,
    })
    if (savedPath) {
      toast.add({ severity: 'success', summary: t('batchDoi.step5.exportSuccess'), life: 4000 })
    }
  } catch (err) {
    toast.add({
      severity: 'error',
      summary: t('batchDoi.step5.exportError', { error: String(err) }),
      life: 8000,
    })
  } finally {
    exporting.value = false
  }
}
</script>
