<template>
  <div class="min-h-screen bg-gray-50 py-8">
    <div class="max-w-5xl mx-auto px-6 space-y-6">
      <!-- Header -->
      <div class="flex items-center justify-between">
        <h1 class="text-2xl font-bold text-gray-900">{{ t('batchDoi.title') }}</h1>
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
          <Step value="1">{{ t('batchDoi.steps.config') }}</Step>
          <Step value="2">{{ t('batchDoi.steps.input') }}</Step>
          <Step value="3">{{ t('batchDoi.steps.overview') }}</Step>
          <Step value="4">{{ t('batchDoi.steps.processing') }}</Step>
          <Step value="5">{{ t('batchDoi.steps.results') }}</Step>
        </StepList>

        <StepPanels>
          <StepPanel value="1" class="pt-6">
            <ConfigStep :env="env" :prefix="selectedPrefix" @next="onStep1Next" />
          </StepPanel>

          <StepPanel value="2" class="pt-6">
            <BatchInputStep
              v-if="activeStep === '2'"
              :env="env"
              :prefix="selectedPrefix"
              @next="onStep2Next"
              @back="activeStep = '1'"
            />
          </StepPanel>

          <StepPanel value="3" class="pt-6">
            <BatchOverviewStep
              v-if="activeStep === '3'"
              :env="env"
              :prefix="selectedPrefix"
              :documents="documents"
              @next="onStep3Next"
              @back="activeStep = '2'"
            />
          </StepPanel>

          <StepPanel value="4" class="pt-6">
            <BatchProcessingStep
              v-if="activeStep === '4'"
              :env="env"
              :prefix="selectedPrefix"
              :documents="documents"
              @next="onStep4Next"
            />
          </StepPanel>

          <StepPanel value="5" class="pt-6">
            <BatchResultsStep
              v-if="activeStep === '5'"
              :documents="documents"
              :env="env"
              @home="router.push('/home')"
            />
          </StepPanel>
        </StepPanels>
      </Stepper>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import Stepper from 'primevue/stepper'
import StepList from 'primevue/steplist'
import Step from 'primevue/step'
import StepPanels from 'primevue/steppanels'
import StepPanel from 'primevue/steppanel'
import Button from 'primevue/button'
import ConfigStep from '../components/doi/steps/ConfigStep.vue'
import BatchInputStep from '../components/doi/steps/batch/BatchInputStep.vue'
import BatchOverviewStep from '../components/doi/steps/batch/BatchOverviewStep.vue'
import BatchProcessingStep from '../components/doi/steps/batch/BatchProcessingStep.vue'
import BatchResultsStep from '../components/doi/steps/batch/BatchResultsStep.vue'
import type { BatchDocument } from '../types/batch'

const { t } = useI18n()
const router = useRouter()

const activeStep = ref('1')
const env = ref<'prod' | 'test'>('test')
const selectedPrefix = ref('')
const documents = ref<BatchDocument[]>([])

function onStep1Next(selectedEnv: 'prod' | 'test', prefix: string) {
  env.value = selectedEnv
  selectedPrefix.value = prefix
  activeStep.value = '2'
}

function onStep2Next(docs: BatchDocument[]) {
  documents.value = docs
  activeStep.value = '3'
}

function onStep3Next(docs: BatchDocument[]) {
  documents.value = docs
  activeStep.value = '4'
}

function onStep4Next(docs: BatchDocument[]) {
  documents.value = docs
  activeStep.value = '5'
}
</script>
