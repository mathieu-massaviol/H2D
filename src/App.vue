<template>
  <div class="flex h-screen overflow-hidden">
    <AppSidebar />
    <main class="flex-1 overflow-auto bg-gray-50">
      <RouterView />
    </main>
  </div>
  <Toast />
  <ConfirmDialog />
</template>

<script setup lang="ts">
import Toast from 'primevue/toast'
import ConfirmDialog from 'primevue/confirmdialog'
import AppSidebar from './components/layout/AppSidebar.vue'
</script>
