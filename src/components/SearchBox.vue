<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { Search } from "lucide-vue-next";

const model = defineModel<string>({ default: "" });
const inputRef = ref<HTMLInputElement>();

function focus() {
  inputRef.value?.focus();
  inputRef.value?.select();
}

onMounted(() => window.addEventListener("focus-search", focus));
onUnmounted(() => window.removeEventListener("focus-search", focus));
</script>

<template>
  <label class="search-box">
    <Search :size="16" />
    <input ref="inputRef" v-model="model" placeholder="搜索画布..." />
    <span>Ctrl+F</span>
  </label>
</template>

<style scoped>
.search-box {
  height: 38px;
  display: grid;
  grid-template-columns: 20px 1fr auto;
  align-items: center;
  gap: 8px;
  padding: 0 11px;
  color: #9ca3af;
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 6px;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}

input {
  min-width: 0;
  color: #374151;
  border: 0;
  outline: 0;
  background: transparent;
}

span {
  font-size: 13px;
  color: #9ca3af;
}
</style>
