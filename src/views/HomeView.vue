<template>
  <div class="min-h-screen bg-gray-50 py-8">
    <div class="max-w-3xl mx-auto px-6 space-y-6">
      <!-- Header -->
      <div>
        <h1 class="text-2xl font-bold text-gray-900">{{ t('home.title') }}</h1>
        <p class="text-gray-600 mt-1">{{ t('home.subtitle') }}</p>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <!-- Create single DOI -->
        <Card>
          <template #title>
            <span class="flex items-center gap-2">
              <i class="pi pi-plus-circle text-primary" />
              {{ t('home.createDoi.title') }}
            </span>
          </template>
          <template #content>
            <div class="flex flex-col gap-4">
              <p class="text-gray-600 text-sm leading-relaxed">
                {{ t('home.createDoi.description') }}
              </p>
              <Button
                :label="t('home.createDoi.title')"
                icon="pi pi-arrow-right"
                iconPos="right"
                @click="router.push('/create-doi')"
              />
            </div>
          </template>
        </Card>

        <!-- List DOIs -->
        <Card>
          <template #title>
            <span class="flex items-center gap-2">
              <i class="pi pi-table text-primary" />
              {{ t('home.doiList.title') }}
            </span>
          </template>
          <template #content>
            <div class="flex flex-col gap-4">
              <p class="text-gray-600 text-sm leading-relaxed">
                {{ t('home.doiList.description') }}
              </p>
              <Button
                :label="t('home.doiList.title')"
                icon="pi pi-arrow-right"
                iconPos="right"
                @click="router.push('/doi-list')"
              />
            </div>
          </template>
        </Card>

        <!-- Batch DOI creation -->
        <Card>
          <template #title>
            <span class="flex items-center gap-2">
              <i class="pi pi-list text-primary" />
              {{ t('home.batchDoi.title') }}
            </span>
          </template>
          <template #content>
            <div class="flex flex-col gap-4">
              <p class="text-gray-600 text-sm leading-relaxed">
                {{ t('home.batchDoi.description') }}
              </p>
              <Button
                :label="t('home.batchDoi.title')"
                icon="pi pi-arrow-right"
                iconPos="right"
                @click="router.push('/batch-doi')"
              />
            </div>
          </template>
        </Card>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import Card from 'primevue/card'
import Button from 'primevue/button'
import Tag from 'primevue/tag'

const { t } = useI18n()
const router = useRouter()
</script>
