<template>
  <div class="min-h-screen bg-gray-50 py-page-y px-page-x space-y-6">
      <!-- Header -->
      <div class="flex items-center justify-between">
        <h1 class="text-2xl font-bold text-gray-900">{{ t('createDoi.title') }}</h1>
        <Button
          :label="t('common.backToHome')"
          icon="pi pi-arrow-left"
          severity="secondary"
          outlined
          size="small"
          @click="router.push('/home')"
        />
      </div>

      <Stepper :value="activeStep" linear>
        <StepList>
          <Step value="1">{{ t('createDoi.steps.config') }}</Step>
          <Step value="2">{{ t('createDoi.steps.selectDocument') }}</Step>
          <Step value="3">{{ t('createDoi.steps.publish') }}</Step>
        </StepList>

        <StepPanels>
          <StepPanel value="1" class="pt-6">
            <ConfigStep
              :prefix="selectedPrefix"
              :suffix-prefix="suffixPrefix"
              @next="onStep1Next"
            />
          </StepPanel>

          <StepPanel value="2" class="pt-6">
            <!-- v-show preserves internal state (metadata + confirm view) when returning from step 3 -->
            <SelectDocumentStep
              v-show="activeStep === '2'"
              :env="env"
              :prefix="selectedPrefix"
              :suffix-prefix="suffixPrefix"
              :hal-id="halId"
              @next="onStep2Next"
              @back="activeStep = '1'"
            />
          </StepPanel>

          <StepPanel value="3" class="pt-6">
            <PublishStep
              v-if="draftDoi && dataciteMetadata && activeStep === '3'"
              :draft-doi="draftDoi"
              :datacite-metadata="dataciteMetadata"
              :hal-id="halId"
              :hal-uri="halUri"
              :env="env"
              @home="router.push('/home')"
              @cancel="router.push('/home')"
              @back="onStep3Back"
            />
          </StepPanel>
        </StepPanels>
      </Stepper>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useEnv } from '../composables/useEnv'
import Stepper from 'primevue/stepper'
import StepList from 'primevue/steplist'
import Step from 'primevue/step'
import StepPanels from 'primevue/steppanels'
import StepPanel from 'primevue/steppanel'
import Button from 'primevue/button'
import ConfigStep from '../components/doi/steps/ConfigStep.vue'
import SelectDocumentStep from '../components/doi/steps/SelectDocumentStep.vue'
import PublishStep from '../components/doi/steps/PublishStep.vue'
import type { HalMetadata } from '../components/doi/steps/SelectDocumentStep.vue'
import type { DataciteMetadata } from '../components/doi/steps/PublishStep.vue'

const { t } = useI18n()
const router = useRouter()

const activeStep = ref('1')
const { env } = useEnv()
const selectedPrefix = ref('')
const suffixPrefix = ref('')
const halId = ref('')
const halUri = ref<string | null>(null)
const draftDoi = ref('')
const dataciteMetadata = ref<DataciteMetadata | null>(null)

function onStep1Next(prefix: string, sp: string) {
  selectedPrefix.value = prefix
  suffixPrefix.value = sp
  activeStep.value = '2'
}

function onStep2Next(id: string, metadata: HalMetadata, doi: string, dcMeta: DataciteMetadata) {
  halId.value = id
  halUri.value = metadata.uri
  draftDoi.value = doi
  dataciteMetadata.value = dcMeta
  activeStep.value = '3'
}

function onStep3Back() {
  draftDoi.value = ''
  dataciteMetadata.value = null
  activeStep.value = '2'
}
</script>
