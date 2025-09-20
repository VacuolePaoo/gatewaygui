<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
</script>

<template>
  <div class="flex h-10 bg-sidebar">
    <p class="items-center justify-center flex px-4">
      {{ t('app.name') }}
    </p>
  </div>
</template>
