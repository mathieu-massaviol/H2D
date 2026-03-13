<template>
  <div class="grid grid-cols-[10rem_1fr] gap-3 px-4 py-2.5">
    <span class="text-sm font-medium text-gray-600 pt-0.5 shrink-0">{{ label }}</span>
    <div v-if="$slots.default">
      <slot />
    </div>
    <span v-else class="text-sm text-gray-700">{{ value }}</span>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  label: string
  value?: string
}>()
</script>
