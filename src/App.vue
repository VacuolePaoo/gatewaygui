<script setup lang="ts">
import { ConfigProvider } from 'reka-ui'
import AppSidebar from '@/components/AppSidebar.vue'
import AppTopbar from '@/components/AppTopbar.vue'
import { TooltipProvider } from '@/components/ui/tooltip'
</script>

<template>
  <ConfigProvider>
    <div class="flex flex-col h-screen">
      <AppTopbar />
      <TooltipProvider class="flex flex-1 overflow-hidden">
        <main class="flex flex-1 overflow-hidden">
          <AppSidebar />
          <section class="bg-background grow flex flex-col overflow-hidden">
            <div class="p-4 flex-1 overflow-auto">
              <router-view v-slot="{ Component }">
                <transition name="page" mode="out-in">
                  <component :is="Component" />
                </transition>
              </router-view>
            </div>
          </section>
        </main>
      </TooltipProvider>
    </div>
  </ConfigProvider>
</template>

<style>
.page-enter-active,
.page-leave-active {
  transition: all 0.3s ease-out;
}

.page-enter-from {
  opacity: 0;
  transform: translateY(10px);
}

.page-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>
