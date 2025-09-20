<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n({ useScope: 'global' })

// 模拟本机信息数据
const hostInfo = {
  hostname: 'MyComputer-PC',
  ip: '192.168.1.105',
  port: '52001',
  status: 'listening',
}
</script>

<template>
  <div class="container mx-auto py-4 max-w-6xl">
    <div class="mb-4">
      <h1 class="text-3xl font-bold">
        {{ t('receive.title') }}
      </h1>
      <p class="text-muted-foreground">
        {{ t('receive.subtitle') }}
      </p>
    </div>

    <div class="flex flex-col lg:flex-row gap-8">
      <!-- 左侧：等待文件接收 (仅在lg及以上屏幕显示) -->
      <div class="hidden lg:flex flex-col items-center justify-center flex-1">
        <img src="@/assets/receive.svg" alt="等待文件接收" class="w-64 h-64">
        <h2 class="text-2xl font-bold mb-2">
          {{ t('receive.waiting.title') }}
        </h2>
        <p class="text-muted-foreground text-center">
          {{ t('receive.waiting.description') }}
        </p>
      </div>

      <!-- 右侧：本机信息 -->
      <div class="flex-1 mt-4">
        <h2 class="text-2xl font-bold mb-8">
          {{ t('receive.host.title') }}
        </h2>
        <div class="space-y-4">
          <div class="flex items-center justify-between p-3 bg-muted rounded-lg">
            <span class="text-muted-foreground">{{ t('receive.host.hostname') }}</span>
            <span class="font-medium">{{ hostInfo.hostname }}</span>
          </div>

          <div class="flex items-center justify-between p-3 bg-muted rounded-lg">
            <span class="text-muted-foreground">{{ t('receive.host.ip') }}</span>
            <span class="font-medium">{{ hostInfo.ip }}</span>
          </div>

          <div class="flex items-center justify-between p-3 bg-muted rounded-lg">
            <span class="text-muted-foreground">{{ t('receive.host.port') }}</span>
            <span class="font-medium">{{ hostInfo.port }}</span>
          </div>

          <div class="flex items-center justify-between p-3 bg-muted rounded-lg">
            <span class="text-muted-foreground">{{ t('receive.host.status') }}</span>
            <span class="font-medium text-green-600">{{ t('receive.host.listening') }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
