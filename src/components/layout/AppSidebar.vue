<template>
  <aside class="flex flex-col w-52 min-h-screen border-r border-gray-200 bg-white shrink-0">
    <div class="flex items-center px-4 py-5 border-b border-gray-200">
      <span class="text-primary font-bold text-lg tracking-tight">hal2datacite</span>
    </div>

    <Menu :model="navItems" class="border-none! shadow-none! w-full! flex-1">
      <template #item="{ item, props }">
        <RouterLink v-if="item.route" v-slot="{ href, navigate, isActive }" :to="item.route" custom>
          <a
            v-bind="props.action"
            :href="href"
            :class="isActive ? 'bg-primary! text-primary-contrast!' : ''"
            @click="navigate"
          >
            <i :class="['pi', item.icon]" />
            <span>{{ item.label }}</span>
          </a>
        </RouterLink>
      </template>
    </Menu>

    <div class="flex items-center justify-center gap-1 px-4 py-3 border-t border-gray-200">
      <button
        v-for="lang in languages"
        :key="lang.code"
        :class="[
          'px-2 py-1 text-xs rounded transition-colors',
          locale === lang.code
            ? 'bg-primary text-primary-contrast font-semibold'
            : 'text-gray-500 hover:text-gray-800 hover:bg-gray-100',
        ]"
        @click="locale = lang.code"
      >
        {{ lang.label }}
      </button>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'
import Menu from 'primevue/menu'

const { t, locale } = useI18n()

const languages = [
  { code: 'fr', label: 'FR' },
  { code: 'en', label: 'EN' },
]

const navItems = computed(() => [
  { route: '/home', icon: 'pi-home', label: t('nav.home') },
  { route: '/credentials', icon: 'pi-key', label: t('nav.credentials') },
])
</script>
