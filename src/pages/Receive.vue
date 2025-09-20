<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { useI18n } from 'vue-i18n'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

const { t } = useI18n({ useScope: 'global' })
</script>

<template>
  <div class="container mx-auto py-4 max-w-6xl">
    <div class="mb-8">
      <h1 class="text-3xl font-bold">
        {{ t('receive.title') }}
      </h1>
      <p class="text-muted-foreground">
        {{ t('receive.subtitle') }}
      </p>
    </div>

    <Card class="mb-8">
      <CardHeader>
        <CardTitle class="text-2xl flex items-center gap-2">
          <Icon icon="ph:link-simple-bold" class="h-6 w-6" />
          {{ t('receive.link.title') }}
        </CardTitle>
        <CardDescription>
          {{ t('receive.link.description') }}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div class="flex flex-col sm:flex-row gap-4">
          <div class="flex-1">
            <Label for="receive-link" class="sr-only">
              {{ t('receive.link.inputLabel') }}
            </Label>
            <Input
              id="receive-link"
              type="text"
              :placeholder="t('receive.link.placeholder')"
              class="w-full"
            />
          </div>
          <Button>
            {{ t('receive.link.button') }}
          </Button>
        </div>
      </CardContent>
    </Card>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <Card class="cursor-pointer hover:bg-accent transition-colors relative overflow-hidden group">
        <div class="absolute right-4 top-4 opacity-10 group-hover:opacity-20 transition-opacity">
          <Icon
            icon="ph:qrcode-fill"
            class="h-24 w-24 text-primary transition-transform duration-300 group-hover:-rotate-12 group-hover:scale-110"
          />
        </div>
        <CardHeader>
          <CardTitle class="text-2xl">
            {{ t('receive.qr.title') }}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <CardDescription class="mb-4">
            {{ t('receive.qr.description') }}
          </CardDescription>
          <div class="bg-white p-4 rounded-lg w-fit">
            <div class="bg-gray-200 border-2 border-dashed rounded-xl w-32 h-32 flex items-center justify-center">
              {{ t('receive.qr.placeholder') }}
            </div>
          </div>
        </CardContent>
      </Card>

      <Card class="cursor-pointer hover:bg-accent transition-colors relative overflow-hidden group">
        <div class="absolute right-4 top-4 opacity-10 group-hover:opacity-20 transition-opacity">
          <Icon
            icon="ph:desktop-fill"
            class="h-24 w-24 text-primary transition-transform duration-300 group-hover:-rotate-12 group-hover:scale-110"
          />
        </div>
        <CardHeader>
          <CardTitle class="text-2xl">
            {{ t('receive.device.title') }}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <CardDescription>
            {{ t('receive.device.description') }}
          </CardDescription>
        </CardContent>
      </Card>
    </div>
  </div>
</template>
