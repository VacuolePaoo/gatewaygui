<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { invoke } from '@tauri-apps/api/core'
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'

const { t } = useI18n()
const router = useRouter()

// 从路由中获取传递的文件列表
const selectedFiles = ref<string[]>([])

// 模拟的设备列表
const devices = ref([
  { id: 1, name: '设备 1', type: '手机', status: 'online' },
  { id: 2, name: '设备 2', type: '电脑', status: 'online' },
  { id: 3, name: '设备 3', type: '平板', status: 'offline' },
])

onMounted(async () => {
  try {
    // 从后端获取文件列表
    const files = await invoke('get_selected_files') as string[]
    if (files && files.length > 0) {
      selectedFiles.value = files
    }
  }
  catch (error) {
    console.error('Error loading files from backend:', error)
  }
})
</script>

<template>
  <div class="container mx-auto py-4 max-w-6xl relative">
    <div class="mb-6 flex justify-between items-center">
      <div>
        <h1 class="text-3xl font-bold">
          {{ t('selectDevice.title') }}
        </h1>
        <p class="text-muted-foreground">
          {{ t('selectDevice.description') }}
        </p>
      </div>
      <Button variant="outline" @click="router.back()">
        <Icon icon="ph:arrow-left" class="mr-2 h-4 w-4" />
        {{ t('selectDevice.back') }}
      </Button>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
      <!-- 已选择的文件列表 -->
      <Card class="md:col-span-1">
        <CardHeader>
          <CardTitle>{{ t('selectDevice.selectedFiles') }}</CardTitle>
          <CardDescription>{{ t('selectDevice.selectedFilesDesc') }}</CardDescription>
        </CardHeader>
        <CardContent>
          <div v-if="selectedFiles.length === 0" class="text-muted-foreground py-4 text-center">
            {{ t('selectDevice.noFiles') }}
          </div>
          <div v-else class="space-y-2 max-h-96 overflow-y-auto">
            <div
              v-for="(file, index) in selectedFiles"
              :key="index"
              class="flex items-center p-2 rounded-md bg-muted"
            >
              <span class="truncate text-sm">{{ file }}</span>
            </div>
          </div>
        </CardContent>
      </Card>

      <!-- 设备选择列表 -->
      <Card class="md:col-span-2">
        <CardHeader>
          <CardTitle>{{ t('selectDevice.availableDevices') }}</CardTitle>
          <CardDescription>{{ t('selectDevice.availableDevicesDesc') }}</CardDescription>
        </CardHeader>
        <CardContent>
          <div class="space-y-4">
            <div
              v-for="device in devices"
              :key="device.id"
              class="flex items-center justify-between p-4 rounded-lg border hover:bg-accent transition-colors"
            >
              <div class="flex items-center space-x-4">
                <div class="w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center">
                  <span class="text-primary font-semibold">
                    {{ device.type === '手机' ? '📱' : device.type === '电脑' ? '💻' : '📱' }}
                  </span>
                </div>
                <div>
                  <h3 class="font-medium">
                    {{ device.name }}
                  </h3>
                  <p class="text-sm text-muted-foreground">
                    {{ device.type }}
                  </p>
                </div>
              </div>
              <div class="flex items-center space-x-4">
                <span
                  class="px-2 py-1 rounded-full text-xs" :class="[
                    device.status === 'online' ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800',
                  ]"
                >
                  {{ device.status === 'online' ? t('selectDevice.online') : t('selectDevice.offline') }}
                </span>
                <Button
                  :disabled="device.status !== 'online'"
                  @click="() => { /* TODO: 实现选择设备逻辑 */ }"
                >
                  {{ t('selectDevice.select') }}
                </Button>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  </div>
</template>
