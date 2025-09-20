<script setup lang="ts">
import type { Language } from '@/lib/config'
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import ThemeSwitch from '@/components/ThemeSwitch.vue'

import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Separator } from '@/components/ui/separator'
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from '@/components/ui/tabs'

import { getLanguageLabel, supportedLanguages } from '@/lib/config'
import { useSettingsStore } from '@/stores/settings'

const { t, locale } = useI18n()
const availableLanguages = ref<Language[]>(supportedLanguages())
const settingsStore = useSettingsStore()

watch(locale, (newLocale, oldLocale) => {
  if (newLocale && newLocale !== oldLocale)
    handleLanguageSelect(newLocale)
}, { immediate: true })

function handleLanguageSelect(newLocale: string) {
  if (!newLocale || !availableLanguages.value.some(sl => sl.value === newLocale))
    return
  settingsStore.setSetting<string>('language', newLocale)
}
</script>

<template>
  <div class="container mx-auto py-8 max-w-6xl">
    <div class="mb-8">
      <h1 class="text-3xl font-bold">
        {{ t('settings.label') }}
      </h1>
      <p class="text-muted-foreground">
        {{ t('settings.description') }}
      </p>
    </div>

    <Tabs default-value="appearance" class="w-full">
      <TabsList class="grid w-full grid-cols-4">
        <TabsTrigger value="core">
          {{ t('settings.tabs.core') }}
        </TabsTrigger>
        <TabsTrigger value="security">
          {{ t('settings.tabs.security') }}
        </TabsTrigger>
        <TabsTrigger value="appearance">
          {{ t('settings.tabs.appearance') }}
        </TabsTrigger>
        <TabsTrigger value="about">
          {{ t('settings.tabs.about') }}
        </TabsTrigger>
      </TabsList>

      <TabsContent value="core" class="mt-6">
        <div class="space-y-8">
          <div class="text-muted-foreground">
            {{ t('settings.tabs.coreDescription') }}
          </div>
        </div>
      </TabsContent>

      <TabsContent value="security" class="mt-6">
        <div class="space-y-8">
          <div class="text-muted-foreground">
            {{ t('settings.tabs.securityDescription') }}
          </div>
        </div>
      </TabsContent>

      <TabsContent value="appearance" class="mt-6">
        <div class="space-y-8">
          <div class="flex items-center space-x-2">
            <Label class="text-lg font-medium" for="theme-switch">{{ t('settings.theme.label') }}</Label>
            <ThemeSwitch />
          </div>
          <div class="flex items-center space-x-2">
            <Label class="text-lg font-medium" for="language-select">{{ t('languages.label') }}</Label>
            <Select id="language-select" v-model="locale">
              <SelectTrigger>
                <SelectValue :placeholder="getLanguageLabel(locale)" />
              </SelectTrigger>
              <SelectContent>
                <SelectGroup>
                  <SelectItem
                    v-for="availableLanguage in availableLanguages"
                    :key="availableLanguage.value"
                    :value="availableLanguage.value"
                  >
                    {{ availableLanguage.label }}
                  </SelectItem>
                </SelectGroup>
              </SelectContent>
            </Select>
          </div>
        </div>
      </TabsContent>

      <TabsContent value="about" class="mt-6">
        <div class="space-y-8">
          <div class="text-muted-foreground">
            {{ t('settings.tabs.aboutDescription') }}
          </div>
        </div>
      </TabsContent>
    </Tabs>
  </div>
</template>
