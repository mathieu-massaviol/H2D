<template>
  <div class="flex flex-col gap-4 max-w-2xl mx-auto pb-4 px-4">
    <!-- Success state -->
    <template v-if="done">
      <Message v-if="halError" severity="warn" :closable="false">
        {{ t('createDoi.step4.halUpdateFailedDetailed') }}
      </Message>
      <div class="flex flex-col items-center gap-4 py-8 text-center">
        <div class="flex items-center justify-center w-16 h-16 rounded-full bg-green-100">
          <i class="pi pi-check text-green-600 text-3xl" />
        </div>
        <h2 class="text-lg font-semibold text-gray-900">{{ t('createDoi.step4.title') }}</h2>

        <div class="flex flex-col gap-1 items-center">
          <span class="text-sm text-gray-600">{{ t('createDoi.step4.doi') }}</span>
          <span class="font-mono text-primary font-semibold text-lg">{{ draftDoi }}</span>
        </div>

        <div class="flex gap-2 mt-2">
          <Button
            :label="t('createDoi.step4.viewOnDatacite')"
            icon="pi pi-external-link"
            iconPos="right"
            severity="secondary"
            outlined
            @click="openDatacite"
          />
          <Button
            v-if="halUri"
            :label="t('createDoi.step4.viewOnHal')"
            icon="pi pi-external-link"
            iconPos="right"
            severity="secondary"
            outlined
            @click="openHal"
          />
        </div>

        <Button
          :label="t('common.backToHome')"
          icon="pi pi-home"
          class="mt-2"
          @click="emit('home')"
        />
      </div>
    </template>

    <!-- Review + publish state -->
    <template v-else>
      <h2 class="text-lg font-semibold text-gray-900">{{ t('createDoi.step3.title') }}</h2>

      <!-- Publish error -->
      <Message v-if="publishError" severity="error" :closable="false">{{ publishError }}</Message>

      <!-- Draft DOI panel -->
      <div class="flex flex-col gap-1">
        <span class="text-sm font-medium text-gray-600">{{ t('createDoi.step3.draftDoi') }}</span>
        <div class="flex items-center gap-3 bg-primary/5 border border-primary/20 rounded-lg px-4 py-3">
          <i class="pi pi-tag text-primary text-lg" />
          <span class="font-mono text-primary font-semibold text-base break-all">{{ draftDoi }}</span>
          <Button icon="pi pi-copy" size="small" text severity="secondary" @click="copyDoi" />
        </div>
      </div>

      <!-- DataCite metadata summary -->
      <div class="flex flex-col gap-2">
        <h3 class="text-sm font-semibold text-gray-900">{{ t('createDoi.step3.dataciteMetadata') }}</h3>

        <div class="flex flex-col divide-y divide-gray-100 border border-gray-200 rounded-lg overflow-hidden bg-white">
          <MetaRow :label="t('createDoi.step3.fields.titles')">
            <ul class="list-none p-0 m-0">
              <li v-for="(title, i) in dataciteMetadata.titles" :key="i" class="text-sm text-gray-700">
                {{ title.title }}
                <span v-if="title.lang" class="text-xs text-gray-400">({{ title.lang }})</span>
              </li>
            </ul>
          </MetaRow>

          <MetaRow :label="t('createDoi.step3.fields.creators')">
            <div class="flex flex-col gap-0.5">
              <span v-for="(c, i) in dataciteMetadata.creators" :key="i" class="text-sm text-gray-700">
                {{ c.name }}
              </span>
            </div>
          </MetaRow>

          <MetaRow
            :label="t('createDoi.step3.fields.type')"
            :value="`${dataciteMetadata.types.resourceTypeGeneral}${dataciteMetadata.types.resourceType ? ' / ' + dataciteMetadata.types.resourceType : ''}`"
          />

          <MetaRow
            :label="t('createDoi.step3.fields.publicationYear')"
            :value="String(dataciteMetadata.publicationYear)"
          />

          <MetaRow :label="t('createDoi.step3.fields.publisher')" :value="dataciteMetadata.publisher.name" />

          <MetaRow
            v-if="dataciteMetadata.language"
            :label="t('createDoi.step3.fields.language')"
            :value="dataciteMetadata.language"
          />

          <MetaRow v-if="dataciteMetadata.subjects?.length" :label="t('createDoi.step3.fields.subjects')">
            <div class="flex flex-wrap gap-1">
              <Tag v-for="(s, i) in dataciteMetadata.subjects" :key="i" :value="s.subject" severity="secondary" />
            </div>
          </MetaRow>

          <MetaRow v-if="dataciteMetadata.descriptions?.length" :label="t('createDoi.step3.fields.descriptions')">
            <p class="text-sm text-gray-700 line-clamp-3">{{ dataciteMetadata.descriptions[0].description }}</p>
          </MetaRow>

          <MetaRow v-if="dataciteMetadata.rightsList?.length" :label="t('createDoi.step3.fields.rights')">
            <a
              v-if="dataciteMetadata.rightsList[0].rightsUri"
              :href="dataciteMetadata.rightsList[0].rightsUri"
              target="_blank"
              class="text-primary text-sm hover:underline"
            >
              {{ dataciteMetadata.rightsList[0].rightsUri }}
            </a>
            <span v-else class="text-sm text-gray-700">{{ dataciteMetadata.rightsList[0].rights }}</span>
          </MetaRow>

          <MetaRow v-if="dataciteMetadata.fundingReferences?.length" :label="t('createDoi.step3.fields.fundingReferences')">
            <div class="flex flex-col gap-1">
              <div v-for="(f, i) in dataciteMetadata.fundingReferences" :key="i" class="text-sm text-gray-700">
                {{ f.funderName }}<span v-if="f.awardNumber"> — {{ f.awardNumber }}</span>
              </div>
            </div>
          </MetaRow>
        </div>
      </div>

      <!-- Actions -->
      <div class="flex justify-between pt-2">
        <div class="flex gap-2">
          <Button
            :label="t('common.back')"
            icon="pi pi-arrow-left"
            severity="secondary"
            outlined
            :disabled="publishing || deleting"
            @click="goBack"
          />
          <Button
            :label="t('common.cancel')"
            severity="secondary"
            outlined
            :loading="deleting"
            :disabled="publishing"
            @click="cancelAndDelete"
          />
        </div>
        <Button
          :label="t('createDoi.step3.publishButton')"
          icon="pi pi-send"
          iconPos="right"
          :loading="publishing"
          :disabled="deleting"
          @click="publish"
        />
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useConfirm } from 'primevue/useconfirm'
import { useToast } from 'primevue/usetoast'
import Button from 'primevue/button'
import Message from 'primevue/message'
import Tag from 'primevue/tag'
import MetaRow from '../../shared/MetaRow.vue'

export interface DataciteMetadata {
  creators: { name: string; nameType: string; givenName?: string; familyName?: string }[]
  titles: { title: string; lang?: string; titleType?: string }[]
  publisher: { name: string }
  publicationYear: number
  subjects?: { subject: string; lang?: string }[]
  language?: string
  types: { resourceTypeGeneral: string; resourceType?: string }
  alternateIdentifiers?: { alternateIdentifier: string; alternateIdentifierType: string }[]
  fundingReferences?: { funderName: string; awardNumber?: string; awardTitle?: string }[]
  descriptions?: { description: string; descriptionType: string; lang?: string }[]
  rightsList?: { rights?: string; rightsUri?: string }[]
  url: string
}

const props = defineProps<{
  draftDoi: string
  dataciteMetadata: DataciteMetadata
  halId: string
  halUri: string | null
  env: 'prod' | 'test'
}>()

const emit = defineEmits<{
  home: []
  cancel: []
  back: []
}>()

const { t } = useI18n()
const confirm = useConfirm()
const toast = useToast()
const publishing = ref(false)
const deleting = ref(false)
const done = ref(false)
const publishError = ref('')
const halError = ref('')

const dataciteBase = props.env === 'test' ? 'https://handle.test.datacite.org' : 'https://doi.org'

async function publish() {
  publishing.value = true
  publishError.value = ''
  halError.value = ''
  try {
    await invoke('publish_doi', { doi: props.draftDoi, env: props.env })
  } catch (err) {
    publishError.value = t('createDoi.step4.errors.publishFailed', { error: String(err) })
    publishing.value = false
    return
  }
  try {
    await invoke('add_doi_to_hal_record', { halId: props.halId, doi: props.draftDoi, env: props.env })
  } catch (err) {
    halError.value = t('createDoi.step4.errors.halUpdateFailed', { error: String(err) })
  }
  publishing.value = false
  done.value = true
}

function cancelAndDelete() {
  confirm.require({
    message: t('createDoi.step3.cancelConfirm', { doi: props.draftDoi }),
    header: t('createDoi.step3.cancelConfirmTitle'),
    icon: 'pi pi-exclamation-triangle',
    rejectProps: { label: t('common.back'), severity: 'secondary', outlined: true },
    acceptProps: { label: t('common.cancel'), severity: 'danger' },
    accept: async () => {
      deleting.value = true
      try {
        await invoke('delete_doi', { doi: props.draftDoi, env: props.env })
      } catch {
        // Best-effort
      } finally {
        deleting.value = false
      }
      emit('cancel')
    },
  })
}

function goBack() {
  // Navigate immediately; delete the draft DOI in the background (best-effort)
  invoke('delete_doi', { doi: props.draftDoi, env: props.env }).catch(() => {})
  emit('back')
}

async function copyDoi() {
  await navigator.clipboard.writeText(props.draftDoi)
  toast.add({
    severity: 'success',
    summary: t('createDoi.step3.doiCopied'),
    life: 3000,
  })
}

function openDatacite() {
  openUrl(`${dataciteBase}/${props.draftDoi}`)
}

function openHal() {
  if (props.halUri) openUrl(props.halUri)
}
</script>
