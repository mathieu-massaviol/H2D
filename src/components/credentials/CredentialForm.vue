<template>
  <form class="flex flex-col gap-4" @submit.prevent="handleSubmit">
    <div class="flex flex-col gap-1">
      <label :for="`cred-${uid}-id`" class="text-sm font-medium text-gray-700">{{ identifierLabel }}</label>
      <InputText :id="`cred-${uid}-id`" v-model="identifier" :placeholder="identifierLabel" class="w-full" />
    </div>

    <div class="flex flex-col gap-1">
      <label :for="`cred-${uid}-pw`" class="text-sm font-medium text-gray-700">{{ passwordLabel }}</label>
      <Password
        :id="`cred-${uid}-pw`"
        v-model="password"
        :placeholder="passwordLabel"
        :feedback="false"
        toggle-mask
        class="w-full"
        input-class="w-full"
      />
    </div>

    <div class="flex gap-2 justify-end">
      <Button
        v-if="showCancel"
        type="button"
        :label="cancelLabel"
        severity="secondary"
        outlined
        @click="emit('cancel')"
      />
      <Button type="submit" :label="saveLabel" :loading="loading" :disabled="!identifier || !password" icon="pi pi-save" />
    </div>
  </form>
</template>

<script setup lang="ts">
import { ref, getCurrentInstance } from 'vue'
import InputText from 'primevue/inputtext'
import Password from 'primevue/password'
import Button from 'primevue/button'

const props = defineProps<{
  identifierLabel: string
  passwordLabel: string
  saveLabel: string
  cancelLabel: string
  showCancel?: boolean
  loading?: boolean
  initialIdentifier?: string
}>()

const emit = defineEmits<{
  submit: [identifier: string, password: string]
  cancel: []
}>()

const uid = getCurrentInstance()?.uid ?? Math.random()
const identifier = ref(props.initialIdentifier ?? '')
const password = ref('')

function handleSubmit() {
  emit('submit', identifier.value, password.value)
}

defineExpose({
  resetPassword: () => {
    password.value = ''
  },
})
</script>
