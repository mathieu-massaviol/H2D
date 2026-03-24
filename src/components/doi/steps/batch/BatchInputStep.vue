<template>
  <div class="flex flex-col gap-4 max-w-2xl mx-auto pb-4 px-4">
    <h2 class="text-lg font-semibold text-gray-900">{{ t('batchDoi.step2.title') }}</h2>
    <p class="text-sm text-gray-600 leading-relaxed">{{ t('batchDoi.step2.instructions') }}</p>

    <div class="flex flex-col gap-1">
      <div class="flex items-center justify-between">
        <label class="text-sm font-medium text-gray-700">{{ t('batchDoi.step2.label') }}</label>
        <Tag
          v-if="lineCount > 0"
          :value="t('batchDoi.step2.lineCount', { count: lineCount })"
          severity="secondary"
        />
      </div>
      <Textarea
        v-model="rawText"
        :placeholder="t('batchDoi.step2.placeholder')"
        :invalid="!!parseError"
        rows="12"
        class="w-full font-mono text-sm"
        auto-resize
      />
      <small v-if="parseError" class="text-red-500">{{ parseError }}</small>
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
        :label="t('batchDoi.step2.loadMetadata')"
        icon="pi pi-arrow-right"
        icon-pos="right"
        @click="submit"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import Button from 'primevue/button'
import Tag from 'primevue/tag'
import Textarea from 'primevue/textarea'
import type { BatchDocument } from '../../../../types/batch'
import { normalizeHalId } from '../../../../utils/hal'

defineProps<{
  env: 'prod' | 'test'
  prefix: string
}>()

const emit = defineEmits<{
  next: [documents: BatchDocument[]]
  back: []
}>()

const { t } = useI18n()

const rawText = ref('')
const parseError = ref('')

const lineCount = computed(() => {
  return rawText.value
    .split('\n')
    .map((l) => l.trim())
    .filter((l) => l.length > 0 && !l.startsWith('#')).length
})

function submit() {
  parseError.value = ''
  const lines = rawText.value
    .split('\n')
    .map((l) => l.trim())
    .filter((l) => l.length > 0 && !l.startsWith('#'))

  if (lines.length === 0) {
    parseError.value = t('batchDoi.step2.errors.noIds')
    return
  }

  const seen = new Set<string>()
  const docs: BatchDocument[] = []
  for (const line of lines) {
    const halId = normalizeHalId(line)
    if (!seen.has(halId)) {
      seen.add(halId)
      docs.push({
        halId,
        status: 'pending',
        selected: true,
        halMetadata: null,
        overviewError: null,
        doi: null,
        halSynced: false,
        processError: null,
      })
    }
  }
  emit('next', docs)
}
</script>
