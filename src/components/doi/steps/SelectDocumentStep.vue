<template>
  <div class="flex flex-col gap-4 max-w-2xl px-8 pb-4">
    <h2 class="text-lg font-semibold text-gray-900">{{ t('createDoi.step2.title') }}</h2>

    <!-- Search view -->
    <template v-if="view === 'search'">
      <div class="flex flex-col gap-1">
        <label class="text-sm font-medium text-gray-700">{{ t('createDoi.step1.halId') }}</label>
        <InputText
          v-model="localHalId"
          :placeholder="t('createDoi.step1.halIdPlaceholder')"
          :invalid="!!halIdError"
          class="w-full"
          @keydown.enter="fetchMetadata"
        />
        <small v-if="halIdError" class="text-red-500">{{ halIdError }}</small>
      </div>

      <div class="flex justify-between pt-2">
        <Button
          :label="t('common.back')"
          icon="pi pi-arrow-left"
          severity="secondary"
          outlined
          @click="emit('back')"
        />
        <Button
          :label="t('createDoi.step1.fetchMetadata')"
          icon="pi pi-search"
          :loading="fetching"
          @click="fetchMetadata"
        />
      </div>
    </template>

    <!-- Metadata summary + confirm view -->
    <template v-else-if="view === 'confirm' && metadata">
      <!-- Existing DOI hard block -->
      <Message v-if="metadata.existing_doi" severity="error" :closable="false">
        {{ t('createDoi.step2.existingDoiError', { doi: metadata.existing_doi }) }}
      </Message>

      <div class="flex flex-col divide-y divide-gray-100 border border-gray-200 rounded-lg overflow-hidden bg-white">
        <MetaRow :label="t('createDoi.step2.fields.halId')" :value="metadata.hal_id" />

        <MetaRow :label="t('createDoi.step2.fields.title')">
          <ul class="list-none p-0 m-0 flex flex-col gap-1">
            <li v-for="(title, idx) in allTitles" :key="idx" class="text-sm text-gray-700">
              {{ title }}
            </li>
          </ul>
        </MetaRow>

        <MetaRow :label="t('createDoi.step2.fields.authors')">
          <div class="flex flex-wrap gap-x-1">
            <span
              v-for="(author, idx) in metadata.authors"
              :key="idx"
              class="inline-flex items-center gap-1 text-sm text-gray-700"
            >
              {{ author.full_name }}
              <a
                v-if="author.orcid"
                :href="author.orcid"
                target="_blank"
                class="text-primary text-xs font-medium hover:underline"
              >ORCID</a>
              <span v-if="idx < metadata.authors.length - 1" class="text-gray-300">,</span>
            </span>
          </div>
        </MetaRow>

        <MetaRow
          v-if="metadata.publication_date"
          :label="t('createDoi.step2.fields.publicationDate')"
          :value="metadata.publication_date"
        />

        <MetaRow
          v-if="metadata.doc_type"
          :label="t('createDoi.step2.fields.docType')"
          :value="metadata.doc_type"
        />

        <MetaRow
          v-if="metadata.languages.length"
          :label="t('createDoi.step2.fields.language')"
          :value="metadata.languages.join(', ')"
        />

        <MetaRow v-if="allKeywords.length" :label="t('createDoi.step2.fields.keywords')">
          <div class="flex flex-wrap gap-1">
            <Tag v-for="(kw, idx) in allKeywords" :key="idx" :value="kw" severity="secondary" />
          </div>
        </MetaRow>

        <MetaRow v-if="allAbstracts.length" :label="t('createDoi.step2.fields.abstract')">
          <div class="text-sm text-gray-700 leading-relaxed">
            <p :class="{ 'line-clamp-4': !showFullAbstract }">{{ allAbstracts[0] }}</p>
            <button
              v-if="allAbstracts[0] && allAbstracts[0].length > 300"
              class="text-primary text-xs mt-1 hover:underline"
              @click="showFullAbstract = !showFullAbstract"
            >
              {{ showFullAbstract ? t('createDoi.step2.showLess') : t('createDoi.step2.showMore') }}
            </button>
          </div>
        </MetaRow>

        <MetaRow v-if="metadata.uri" :label="t('createDoi.step2.fields.uri')">
          <a :href="metadata.uri" target="_blank" class="text-primary text-sm hover:underline break-all">
            {{ metadata.uri }}
          </a>
        </MetaRow>
      </div>

      <div class="flex justify-between pt-2">
        <Button
          :label="t('common.back')"
          icon="pi pi-arrow-left"
          severity="secondary"
          outlined
          :disabled="creatingDraft"
          @click="view = 'search'"
        />
        <Button
          :label="t('common.confirm')"
          icon="pi pi-check"
          iconPos="right"
          :loading="creatingDraft"
          :disabled="!!metadata.existing_doi"
          @click="confirmAndCreateDraft"
        />
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { useToast } from 'primevue/usetoast'
import InputText from 'primevue/inputtext'
import Button from 'primevue/button'
import Message from 'primevue/message'
import Tag from 'primevue/tag'
import MetaRow from '../../shared/MetaRow.vue'
import type { DataciteMetadata } from './PublishStep.vue'

export interface HalAuthor {
  full_name: string
  first_name: string | null
  last_name: string | null
  orcid: string | null
  affiliations: string[]
}

export interface HalMetadata {
  hal_id: string
  title: string[]
  authors: HalAuthor[]
  publication_date: string | null
  doc_type: string | null
  abstracts: string[]
  keywords: string[]
  languages: string[]
  existing_doi: string | null
  uri: string | null
  license: string | null
  titles_by_lang: Record<string, string[]>
  abstracts_by_lang: Record<string, string[]>
  keywords_by_lang: Record<string, string[]>
  anr_project_titles: string[]
  anr_project_references: string[]
}

const props = defineProps<{
  env: 'prod' | 'test'
  prefix: string
  halId: string
}>()

const emit = defineEmits<{
  next: [halId: string, metadata: HalMetadata, doi: string, dataciteMetadata: DataciteMetadata]
  back: []
}>()

const { t } = useI18n()
const toast = useToast()

const view = ref<'search' | 'confirm'>('search')
const localHalId = ref(props.halId)
const metadata = ref<HalMetadata | null>(null)
const halIdError = ref('')
const fetching = ref(false)
const creatingDraft = ref(false)
const showFullAbstract = ref(false)

function multilang(byLang: Record<string, string[]>, fallback: string[]): string[] {
  const values = Object.values(byLang)
  return values.length > 0 ? values.flat() : fallback
}

const allTitles = computed(() =>
  metadata.value ? multilang(metadata.value.titles_by_lang, metadata.value.title) : [],
)
const allKeywords = computed(() =>
  metadata.value ? multilang(metadata.value.keywords_by_lang, metadata.value.keywords) : [],
)
const allAbstracts = computed(() =>
  metadata.value ? multilang(metadata.value.abstracts_by_lang, metadata.value.abstracts) : [],
)

async function fetchMetadata() {
  halIdError.value = ''
  if (!localHalId.value.trim()) {
    halIdError.value = t('createDoi.step1.errors.halIdRequired')
    return
  }
  fetching.value = true
  try {
    metadata.value = await invoke<HalMetadata>('get_hal_metadata', {
      halId: localHalId.value.trim(),
      env: props.env,
    })
    showFullAbstract.value = false
    view.value = 'confirm'
  } catch (err) {
    toast.add({
      severity: 'error',
      summary: t('createDoi.step1.errors.fetchFailed', { error: String(err) }),
      life: 8000,
    })
  } finally {
    fetching.value = false
  }
}

async function confirmAndCreateDraft() {
  if (!metadata.value) return
  creatingDraft.value = true
  try {
    const [dataciteMetadata, doi] = await Promise.all([
      invoke<DataciteMetadata>('map_hal_to_datacite', { halMetadata: metadata.value }),
      invoke<string>('create_doi', {
        halMetadata: metadata.value,
        prefix: props.prefix,
        env: props.env,
        publish: false,
      }),
    ])
    emit('next', localHalId.value.trim(), metadata.value, doi, dataciteMetadata)
  } catch (err) {
    toast.add({
      severity: 'error',
      summary: t('createDoi.step3.errors.createFailed', { error: String(err) }),
      life: 8000,
    })
  } finally {
    creatingDraft.value = false
  }
}
</script>
