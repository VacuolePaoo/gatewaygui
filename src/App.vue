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
              <router-view />
            </div>
          </section>
        </main>
      </TooltipProvider>
    </div>
  </ConfigProvider>
</template>
