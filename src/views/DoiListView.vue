<template>
  <div class="min-h-screen bg-gray-50 py-page-y px-page-x space-y-6">
      <!-- Header -->
      <div class="flex items-center gap-4">
        <Button
          icon="pi pi-arrow-left"
          severity="secondary"
          text
          @click="router.push('/home')"
        />
        <div>
          <h1 class="text-2xl font-bold text-gray-900">{{ t('doiList.title') }}</h1>
          <p class="text-gray-600 mt-1">{{ t('doiList.subtitle') }}</p>
        </div>
      </div>

      <!-- Controls -->
      <div class="flex flex-wrap items-end gap-4">
        <div class="flex flex-col gap-1">
          <label class="text-sm font-medium text-gray-700">{{ t('doiList.environment') }}</label>
          <SelectButton
            v-model="env"
            :options="envOptions"
            optionLabel="label"
            optionValue="value"
            :allowEmpty="false"
          />
        </div>
        <Button
          :label="t('doiList.fetch')"
          icon="pi pi-refresh"
          :loading="loading"
          @click="fetchDois(1)"
        />
      </div>

      <!-- Error -->
      <Message v-if="error" severity="error" :closable="false">{{ error }}</Message>

      <!-- Results / Empty state -->
      <template v-if="fetched && !loading">
        <template v-if="rows.length > 0">
          <div class="flex items-center justify-between">
            <span class="text-sm text-gray-600">
              {{ t('doiList.total', { count: meta.total }) }}
            </span>
          </div>

          <DataTable
            :value="rows"
            :rows="pageSize"
            :rowHover="true"
            striped-rows
            class="cursor-pointer"
            @row-click="onRowClick"
          >
            <Column field="doi" :header="t('doiList.columns.doi')" style="min-width: 13rem">
              <template #body="{ data }">
                <span class="font-mono text-sm text-primary truncate block">{{ data.doi }}</span>
              </template>
            </Column>
            <Column field="halId" :header="t('doiList.columns.halId')" style="width: 10rem">
              <template #body="{ data }">
                <span class="font-mono text-sm text-gray-700">{{ data.halId ?? '—' }}</span>
              </template>
            </Column>
            <Column field="title" :header="t('doiList.columns.title')">
              <template #body="{ data }">
                <span class="text-sm text-gray-900 line-clamp-2">{{ data.title || '—' }}</span>
              </template>
            </Column>
            <Column field="state" :header="t('doiList.columns.state')" style="width: 7rem">
              <template #body="{ data }">
                <Tag
                  :value="t(`doiList.states.${data.state}`)"
                  :severity="stateSeverity(data.state)"
                />
              </template>
            </Column>
            <Column :header="t('doiList.columns.halSync')" style="width: 10rem">
              <template #body="{ data }">
                <div class="flex items-center gap-2" @click.stop>
                  <!-- Only findable DOIs with a HAL ID are checked -->
                  <template v-if="data.halId && data.state === 'findable'">
                    <i v-if="halSyncMap[data.doi] === 'loading'" class="pi pi-spin pi-spinner text-gray-400 text-base" />
                    <template v-else-if="halSyncMap[data.doi] === true">
                      <i class="pi pi-check-circle text-green-600 text-base" />
                      <span class="text-xs text-green-700">{{ t('doiList.halSync.synced') }}</span>
                    </template>
                    <template v-else-if="halSyncMap[data.doi] === false">
                      <Button
                        :label="t('doiList.halSync.addToHal')"
                        icon="pi pi-upload"
                        size="small"
                        severity="warn"
                        outlined
                        :loading="syncingDoi === data.doi"
                        :disabled="!!syncingDoi && syncingDoi !== data.doi"
                        @click="addToHal(data)"
                      />
                    </template>
                    <span v-else class="text-xs text-gray-400">—</span>
                  </template>
                </div>
              </template>
            </Column>
          </DataTable>

          <Paginator
            v-if="meta.totalPages > 1"
            :rows="pageSize"
            :totalRecords="meta.total"
            :first="(meta.page - 1) * pageSize"
            @page="onPageChange"
          />
        </template>

        <div v-else class="flex flex-col items-center gap-3 py-16 text-gray-500">
          <i class="pi pi-inbox text-4xl" />
          <span class="text-sm">{{ t('doiList.empty') }}</span>
        </div>
      </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useToast } from 'primevue/usetoast'
import Button from 'primevue/button'
import SelectButton from 'primevue/selectbutton'
import Message from 'primevue/message'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import Tag from 'primevue/tag'
import Paginator from 'primevue/paginator'

interface DoiListItem {
  doi: string
  title: string
  state: string
  publicationYear: number | null
  resourceTypeGeneral: string | null
  halId: string | null
}

interface DoiListMeta {
  total: number
  totalPages: number
  page: number
}

interface DoiListResponse {
  items: DoiListItem[]
  meta: DoiListMeta
}

const { t } = useI18n()
const router = useRouter()
const toast = useToast()

const env = ref<'prod' | 'test'>('test')
const loading = ref(false)
const error = ref('')
const fetched = ref(false)
const rows = ref<DoiListItem[]>([])
const pageSize = 25
const meta = ref<DoiListMeta>({ total: 0, totalPages: 0, page: 1 })
// doi → true (synced) | false (not synced) | 'loading' | undefined (check failed / not applicable)
const halSyncMap = ref<Record<string, boolean | 'loading'>>({})
const syncingDoi = ref<string | null>(null)

const envOptions = [
  { label: t('doiList.test'), value: 'test' },
  { label: t('doiList.prod'), value: 'prod' },
]

async function fetchDois(page: number) {
  loading.value = true
  error.value = ''
  halSyncMap.value = {}
  try {
    const result = await invoke<DoiListResponse>('get_datacite_dois', {
      env: env.value,
      page,
      pageSize,
    })
    rows.value = result.items
    meta.value = result.meta
    fetched.value = true
  } catch (err) {
    error.value = t('doiList.errors.fetchFailed', { error: String(err) })
    return
  } finally {
    loading.value = false
  }

  checkHalSync()
}

function checkHalSync() {
  const toCheck = rows.value.filter(row => row.halId && row.state === 'findable')
  if (toCheck.length === 0) return

  // Mark each eligible row as loading individually
  const init: Record<string, 'loading'> = {}
  for (const row of toCheck) init[row.doi] = 'loading'
  halSyncMap.value = init

  // Update each row as its check completes — no waiting for the slowest
  for (const row of toCheck) {
    invoke<{ existing_doi: string | null }>('get_hal_metadata', {
      halId: row.halId!,
      env: env.value,
    })
      .then(metadata => {
        const synced = metadata.existing_doi?.toLowerCase() === row.doi.toLowerCase()
        halSyncMap.value = { ...halSyncMap.value, [row.doi]: synced }
      })
      .catch(() => {
        // Remove from map on error — column shows '—'
        const { [row.doi]: _, ...rest } = halSyncMap.value
        halSyncMap.value = rest
      })
  }
}

async function addToHal(row: DoiListItem) {
  if (!row.halId) return
  syncingDoi.value = row.doi
  try {
    await invoke('add_doi_to_hal_record', { halId: row.halId, doi: row.doi, env: env.value })
    halSyncMap.value = { ...halSyncMap.value, [row.doi]: true }
    toast.add({ severity: 'success', summary: t('doiList.halSync.addSuccess'), life: 4000 })
  } catch (err) {
    toast.add({
      severity: 'error',
      summary: t('doiList.halSync.addFailed', { error: String(err) }),
      life: 8000,
    })
  } finally {
    syncingDoi.value = null
  }
}

function onPageChange(event: { page: number }) {
  fetchDois(event.page + 1)
}

function stateSeverity(state: string): 'success' | 'secondary' | 'warn' {
  if (state === 'findable') return 'success'
  if (state === 'registered') return 'warn'
  return 'secondary'
}

onMounted(() => fetchDois(1))

function onRowClick(event: { data: DoiListItem }) {
  const base = env.value === 'test' ? 'https://handle.test.datacite.org' : 'https://doi.org'
  openUrl(`${base}/${event.data.doi}`)
}
</script>
