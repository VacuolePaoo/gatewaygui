<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { open } from '@tauri-apps/plugin-dialog'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'

const { t } = useI18n()

// 文件列表
const selectedFiles = ref<string[]>([])

// 选择文件
async function selectFiles() {
  try {
    const selected = await open({
      multiple: true,
      directory: false,
      title: t('send.selectFiles'),
    })

    if (selected) {
      if (Array.isArray(selected)) {
        selectedFiles.value = [...selectedFiles.value, ...selected]
      }
      else {
        selectedFiles.value = [...selectedFiles.value, selected]
      }
      // 去重
      selectedFiles.value = [...new Set(selectedFiles.value)]
    }
  }
  catch (error) {
    console.error('Error selecting files:', error)
  }
}

// 选择文件夹
async function selectFolders() {
  try {
    const selected = await open({
      multiple: true,
      directory: true,
      title: t('send.selectFolders'),
    })

    if (selected) {
      if (Array.isArray(selected)) {
        selectedFiles.value = [...selectedFiles.value, ...selected]
      }
      else {
        selectedFiles.value = [...selectedFiles.value, selected]
      }
      // 去重
      selectedFiles.value = [...new Set(selectedFiles.value)]
    }
  }
  catch (error) {
    console.error('Error selecting folders:', error)
  }
}

// 移除选中的文件/文件夹
function removeFile(index: number) {
  selectedFiles.value.splice(index, 1)
}

// 清空所有选择
function clearAll() {
  selectedFiles.value = []
}
</script>

<template>
  <div class="container mx-auto py-4 max-w-6xl">
    <div class="mb-8">
      <h1 class="text-3xl font-bold">
        {{ t('send.title') }}
      </h1>
      <p class="text-muted-foreground">
        {{ t('send.description') }}
      </p>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <!-- 选择区域 -->
      <Card>
        <CardHeader>
          <CardTitle>{{ t('send.select.title') }}</CardTitle>
          <CardDescription>{{ t('send.select.description') }}</CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="flex flex-col sm:flex-row gap-4">
            <Button class="flex-1" @click="selectFiles">
              <Icon icon="ph:file-plus" class="mr-2 h-4 w-4" />
              {{ t('send.select.files') }}
            </Button>
            <Button variant="secondary" class="flex-1" @click="selectFolders">
              <Icon icon="ph:folder-plus" class="mr-2 h-4 w-4" />
              {{ t('send.select.folders') }}
            </Button>
          </div>
        </CardContent>
      </Card>

      <!-- 已选择文件列表 -->
      <Card>
        <CardHeader>
          <CardTitle>{{ t('send.selected.title') }}</CardTitle>
          <CardDescription>{{ t('send.selected.description') }}</CardDescription>
        </CardHeader>
        <CardContent>
          <div v-if="selectedFiles.length === 0" class="text-muted-foreground text-center py-8">
            {{ t('send.selected.empty') }}
          </div>
          <div v-else>
            <div class="flex flex-wrap gap-2 mb-4">
              <Badge 
                v-for="(file, index) in selectedFiles" 
                :key="index" 
                variant="secondary" 
                class="pr-2 py-1.5 max-w-full"
              >
                <span class="truncate max-w-[160px]">{{ file }}</span>
                <button 
                  @click="removeFile(index)"
                  class="ml-1 hover:bg-secondary-foreground/10 rounded-full p-0.5"
                  :aria-label="t('send.selected.remove')"
                >
                  <Icon icon="ph:x" class="h-4 w-4" />
                </button>
              </Badge>
            </div>
            <Button 
              variant="outline" 
              size="sm" 
              @click="clearAll"
              :disabled="selectedFiles.length === 0"
            >
              <Icon icon="ph:trash" class="mr-2 h-4 w-4" />
              {{ t('send.selected.clear') }}
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>

    <!-- 操作按钮 -->
    <div class="mt-6 flex justify-end">
      <Button :disabled="selectedFiles.length === 0">
        <Icon icon="ph:paper-plane-right" class="mr-2 h-4 w-4" />
        {{ t('send.send') }}
      </Button>
    </div>
  </div>
</template>