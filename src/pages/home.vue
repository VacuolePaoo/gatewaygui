<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'

const { t } = useI18n({ useScope: 'global' })

// 根据当前时间返回问候语
const greeting = computed(() => {
  const hour = new Date().getHours()
  if (hour < 12) {
    return t('home.greeting.morning')
  }
  else if (hour < 18) {
    return t('home.greeting.afternoon')
  }
  else {
    return t('home.greeting.evening')
  }
})
</script>

<template>
  <div class="container mx-auto py-8 max-w-6xl">
    <div class="mb-8">
      <h1 class="text-3xl font-bold">
        {{ greeting }}
      </h1>
      <p class="text-muted-foreground">
        {{ t('home.subtitle') }}
      </p>
    </div>
    <!-- Bento Grid Layout -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6 auto-rows-fr">
      <!-- 发送文件卡片 - 大卡片 -->
      <Card class="flex flex-col md:row-span-2 cursor-pointer hover:bg-accent transition-colors relative overflow-hidden group">
        <div class="absolute right-4 top-4 opacity-10 group-hover:opacity-20 transition-opacity">
          <Icon
            icon="ph:paper-plane-right-fill"
            class="h-24 w-24 text-primary fly-plane"
          />
        </div>
        <CardHeader>
          <CardTitle class="text-2xl">
            {{ t('home.send.title') }}
          </CardTitle>
        </CardHeader>
        <CardContent class="flex flex-col flex-1">
          <CardDescription class="mb-2">
            {{ t('home.send.description') }}
          </CardDescription>
          <p class="text-sm text-muted-foreground flex-1">
            {{ t('home.send.details') }}
          </p>
        </CardContent>
      </Card>

      <!-- 接收文件卡片 -->
      <Card class="flex flex-col cursor-pointer hover:bg-accent transition-colors relative overflow-hidden group">
        <div class="absolute right-4 top-4 opacity-10 group-hover:opacity-20 transition-opacity">
          <Icon
            icon="ph:download-fill"
            class="h-24 w-24 text-primary transition-transform duration-300 group-hover:-rotate-12 group-hover:scale-110"
          />
        </div>
        <CardHeader>
          <CardTitle class="text-2xl">
            {{ t('home.receive.title') }}
          </CardTitle>
        </CardHeader>
        <CardContent class="flex flex-col flex-1">
          <CardDescription class="mb-2">
            {{ t('home.receive.description') }}
          </CardDescription>
        </CardContent>
      </Card>

      <!-- 目录挂载卡片 -->
      <Card class="flex flex-col cursor-pointer hover:bg-accent transition-colors relative overflow-hidden group">
        <div class="absolute right-4 top-4 opacity-10 group-hover:opacity-20 transition-opacity">
          <Icon
            icon="ph:folder-fill"
            class="h-24 w-24 text-primary transition-transform duration-300 group-hover:-rotate-12 group-hover:scale-110"
          />
        </div>
        <CardHeader>
          <CardTitle class="text-2xl">
            {{ t('home.mount.title') }}
          </CardTitle>
        </CardHeader>
        <CardContent class="flex flex-col flex-1">
          <CardDescription class="mb-2">
            {{ t('home.mount.description') }}
          </CardDescription>
        </CardContent>
      </Card>
    </div>
  </div>
</template>

<style scoped>
@keyframes flyRightAndLeft {
  0% {
    transform: translateX(0);
    opacity: 1;
  }
  40% {
    transform: translateX(120px);
    opacity: 1;
  }
  50% {
    transform: translateX(150px);
    opacity: 0;
  }
  51% {
    transform: translateX(-150px);
    opacity: 0;
  }
  100% {
    transform: translateX(0);
  }
}

.fly-plane {
  transition: opacity 0.3s ease;
}

.group:hover .fly-plane {
  animation: flyRightAndLeft 1.3s cubic-bezier(0.4, 0, 0.2, 1);
}
</style>