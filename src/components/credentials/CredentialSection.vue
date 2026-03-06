<template>
  <Card>
    <template #title>
      <span class="flex items-center gap-2 text-lg font-semibold">
        {{ title }}
        <Tag v-if="displayValue" :value="configuredText" severity="success" icon="pi pi-check" />
        <Tag v-else :value="notConfiguredText" severity="secondary" />
      </span>
    </template>

    <template #content>
      <!-- Display mode: credentials exist and not editing -->
      <div v-if="displayValue && !editing" class="flex items-center justify-between">
        <div class="flex items-center gap-2 text-gray-700">
          <i class="pi pi-user text-gray-400" />
          <span class="font-mono">{{ displayValue }}</span>
        </div>
        <div class="flex gap-2">
          <Button
            :label="editLabel"
            icon="pi pi-pencil"
            severity="secondary"
            outlined
            size="small"
            @click="editing = true"
          />
          <Button
            :label="deleteLabel"
            icon="pi pi-trash"
            severity="danger"
            outlined
            size="small"
            :loading="deleting"
            @click="emit('delete')"
          />
        </div>
      </div>

      <!-- Form mode: no credentials or editing -->
      <CredentialForm
        v-else
        ref="formRef"
        :identifier-label="identifierLabel"
        :password-label="passwordLabel"
        :save-label="saveLabel"
        :cancel-label="cancelLabel"
        :show-cancel="editing"
        :loading="saving"
        :initial-identifier="displayValue ?? undefined"
        @submit="(id, pw) => emit('save', id, pw)"
        @cancel="editing = false"
      />
    </template>
  </Card>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import Card from 'primevue/card'
import Button from 'primevue/button'
import Tag from 'primevue/tag'
import CredentialForm from './CredentialForm.vue'

defineProps<{
  title: string
  configuredText: string
  notConfiguredText: string
  editLabel: string
  deleteLabel: string
  cancelLabel: string
  saveLabel: string
  identifierLabel: string
  passwordLabel: string
  displayValue: string | null
  saving?: boolean
  deleting?: boolean
}>()

const emit = defineEmits<{
  save: [identifier: string, password: string]
  delete: []
}>()

const editing = ref(false)
const formRef = ref<InstanceType<typeof CredentialForm>>()

defineExpose({
  resetEditing: () => {
    editing.value = false
    formRef.value?.resetPassword()
  },
})
</script>
