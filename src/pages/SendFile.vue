<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { onMounted, onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Textarea } from '@/components/ui/textarea'

const { t } = useI18n()

// 文件列表
const selectedFiles = ref<string[]>([])

// 文本内容
const textContent = ref('')

// 拖拽状态
const isDragOver = ref(false)

// 拖拽监听器
let unlistenDragEnter: (() => void) | null = null
let unlistenDragOver: (() => void) | null = null
let unlistenDragDrop: (() => void) | null = null
let unlistenDragLeave: (() => void) | null = null

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

// 添加文本内容
function addText() {
  if (textContent.value.trim()) {
    // 为文本内容创建一个特殊标识
    const textEntry = `[TEXT] ${textContent.value.trim()}`
    selectedFiles.value = [...selectedFiles.value, textEntry]
    textContent.value = ''
  }
}

// 移除选中的文件/文件夹/文本
function removeFile(index: number) {
  selectedFiles.value.splice(index, 1)
}

// 清空所有选中的文件/文件夹/文本
function clearAllFiles() {
  selectedFiles.value = []
}

// 初始化拖拽监听
onMounted(async () => {
  unlistenDragEnter = await listen('tauri://drag-enter', () => {
    isDragOver.value = true
  })

  unlistenDragOver = await listen('tauri://drag-over', () => {
    isDragOver.value = true
  })

  unlistenDragDrop = await listen('tauri://drag-drop', (event) => {
    isDragOver.value = false
    // 正确处理事件payload - 它是一个包含paths和position属性的对象
    if (event.payload) {
      const payload = event.payload as { paths: string[], position: { x: number, y: number } }
      if (payload.paths && Array.isArray(payload.paths)) {
        selectedFiles.value = [...selectedFiles.value, ...payload.paths]
        // 去重
        selectedFiles.value = [...new Set(selectedFiles.value)]
      }
    }
  })

  unlistenDragLeave = await listen('tauri://drag-leave', () => {
    isDragOver.value = false
  })
})

// 清理监听器
onUnmounted(() => {
  if (unlistenDragEnter)
    unlistenDragEnter()
  if (unlistenDragOver)
    unlistenDragOver()
  if (unlistenDragDrop)
    unlistenDragDrop()
  if (unlistenDragLeave)
    unlistenDragLeave()
})
</script>

<template>
  <div class="container mx-auto py-4 max-w-6xl relative">
    <div class="mb-4">
      <h1 class="text-3xl font-bold">
        {{ t('send.title') }}
      </h1>
      <p class="text-muted-foreground">
        {{ t('send.description') }}
      </p>
    </div>

    <!-- 操作按钮 -->
    <div class="flex flex-wrap gap-3 mb-2">
      <Button @click="selectFiles">
        <Icon icon="ph:file-plus" class="mr-2 h-4 w-4" />
        {{ t('send.select.files') }}
      </Button>
      <Button variant="outline" @click="selectFolders">
        <Icon icon="ph:folder-plus" class="mr-2 h-4 w-4" />
        {{ t('send.select.folders') }}
      </Button>
      <Dialog>
        <DialogTrigger as-child>
          <Button variant="outline">
            <Icon icon="ph:text-align-left" class="mr-2 h-4 w-4" />
            {{ t('send.select.text') }}
          </Button>
        </DialogTrigger>
        <DialogContent class="sm:max-w-[425px]">
          <DialogHeader>
            <DialogTitle>{{ t('send.textDialog.title') }}</DialogTitle>
            <DialogDescription>
              {{ t('send.textDialog.description') }}
            </DialogDescription>
          </DialogHeader>
          <div class="grid gap-4 py-4">
            <Textarea
              v-model="textContent"
              :placeholder="t('send.textDialog.placeholder')"
              class="min-h-[120px]"
            />
          </div>
          <DialogFooter>
            <Button type="submit" @click="addText">
              {{ t('send.textDialog.add') }}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <div class="flex-grow" />
      <Button
        variant="destructive"
        :disabled="selectedFiles.length === 0"
        @click="clearAllFiles"
      >
        <Icon icon="ph:trash" class="mr-2 h-4 w-4" />
        {{ t('send.clearAll') }}
      </Button>
    </div>

    <!-- 已选择文件列表滚动区域 -->
    <ScrollArea
      class="h-[calc(100vh-240px)] rounded-md border p-2"
    >
      <div
        :class="{ 'border-2 border-dashed border-primary rounded-lg': isDragOver }"
        class="h-full w-full"
      >
        <div
          v-if="selectedFiles.length === 0 && !isDragOver"
          class="text-muted-foreground text-center py-8"
        >
          {{ t('send.selected.empty') }}
        </div>
        <div
          v-else-if="isDragOver"
          class="text-primary text-center py-8 flex flex-col items-center justify-center"
        >
          <Icon icon="ph:upload-duotone" class="h-12 w-12 mb-2" />
          <span class="text-lg">{{ t('send.drag.hover') }}</span>
        </div>
        <div v-else class="space-y-3">
          <div
            v-for="(file, index) in selectedFiles"
            :key="index"
            class="flex items-center justify-between p-3 rounded-lg border bg-card text-card-foreground shadow-sm hover:bg-accent hover:text-accent-foreground transition-colors"
          >
            <div class="flex items-center min-w-0 flex-1">
              <Icon
                v-if="file.startsWith('[TEXT]')"
                icon="ph:article"
                class="h-5 w-5 mr-3 flex-shrink-0 text-muted-foreground"
              />
              <Icon
                v-else-if="file.includes('.') && !file.includes('\\') && !file.includes('/')"
                icon="ph:file"
                class="h-5 w-5 mr-3 flex-shrink-0 text-muted-foreground"
              />
              <Icon
                v-else
                icon="ph:folder"
                class="h-5 w-5 mr-3 flex-shrink-0 text-muted-foreground"
              />
              <span class="truncate text-sm">{{ file.startsWith('[TEXT]') ? file.substring(7) : file }}</span>
            </div>
            <button
              class="ml-2 p-1 rounded-full hover:bg-muted text-muted-foreground hover:text-foreground"
              :aria-label="t('send.selected.remove')"
              @click="removeFile(index)"
            >
              <Icon icon="ph:x" class="h-4 w-4" />
            </button>
          </div>
        </div>
      </div>
    </ScrollArea>

    <!-- 悬浮操作按钮 -->
    <div class="fixed bottom-6 right-6">
      <Button size="lg" :disabled="selectedFiles.length === 0" class="shadow-lg">
        <Icon icon="ph:paper-plane-right" class="mr-2 h-5 w-5" />
        {{ t('send.next') }}
      </Button>
    </div>
  </div>
</template>
