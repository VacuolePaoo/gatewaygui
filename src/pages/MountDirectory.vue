<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { open } from '@tauri-apps/plugin-dialog'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'

const { t } = useI18n()

// 模拟已挂载的目录数据
const mountedDirectories = ref<string[]>([
  '/home/user/Documents',
  '/home/user/Downloads',
  '/home/user/Pictures',
  '/mnt/shared/projects',
])

async function addMountPoint() {
  try {
    // 打开文件夹选择对话框
    const selected = await open({
      directory: true,
      multiple: false,
    })

    // 如果用户选择了文件夹，则添加到目录列表
    if (selected && typeof selected === 'string') {
      mountedDirectories.value.push(selected)
    }

    // 如果用户取消选择，selected 将为 null，不执行任何操作
  }
  catch (error) {
    console.error('选择文件夹时出错:', error)
  }
}

async function editDirectory(directory: string) {
  try {
    // 打开文件夹选择对话框
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: directory,
    })

    // 如果用户选择了文件夹，则更新目录
    if (selected && typeof selected === 'string') {
      const index = mountedDirectories.value.indexOf(directory)
      if (index !== -1) {
        mountedDirectories.value[index] = selected
      }
    }

    // 如果用户取消选择，selected 将为 null，不执行任何操作
  }
  catch (error) {
    console.error('选择文件夹时出错:', error)
  }
}

function removeDirectory(index: number) {
  // TODO: 实现删除目录功能
  console.warn('删除目录功能待实现，索引:', index)
  mountedDirectories.value.splice(index, 1)
}
</script>

<template>
  <div class="container mx-auto py-4 max-w-6xl relative">
    <div class="mb-4 flex justify-between items-center">
      <div>
        <h1 class="text-3xl font-bold">
          {{ t('home.mount.title') }}
        </h1>
        <p class="text-muted-foreground">
          {{ t('home.mount.description') }}
        </p>
      </div>
      <Button @click="addMountPoint">
        <Icon icon="ph:plus" class="mr-2 h-5 w-5" />
        {{ t('mount.addMountPoint') }}
      </Button>
    </div>

    <!-- 已挂载目录列表滚动区域 -->
    <ScrollArea class="h-[calc(100vh-200px)] rounded-md border p-2">
      <div class="h-full w-full">
        <div
          v-if="mountedDirectories.length === 0"
          class="text-muted-foreground text-center py-8"
        >
          {{ t('mount.noMountedDirectories') }}
        </div>
        <div v-else class="space-y-3">
          <div
            v-for="(directory, index) in mountedDirectories"
            :key="index"
            class="flex items-center justify-between p-3 rounded-lg border bg-card text-card-foreground hover:bg-accent hover:text-accent-foreground transition-colors group"
          >
            <div class="flex items-center min-w-0 flex-1">
              <Icon
                icon="ph:folder"
                class="h-5 w-5 mr-3 flex-shrink-0 text-muted-foreground"
              />
              <span class="truncate text-sm">{{ directory }}</span>
            </div>
            <div class="flex space-x-1 opacity-0 group-hover:opacity-100 transition-opacity">
              <button
                class="p-1 rounded-full hover:bg-muted text-muted-foreground hover:text-foreground"
                :aria-label="t('mount.edit')"
                @click="editDirectory(directory)"
              >
                <Icon icon="ph:folder-open-duotone" class="h-4 w-4" />
              </button>
              <button
                class="p-1 rounded-full hover:bg-muted text-muted-foreground hover:text-foreground"
                :aria-label="t('mount.remove')"
                @click="removeDirectory(index)"
              >
                <Icon icon="ph:trash" class="h-4 w-4" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </ScrollArea>
  </div>
</template>
