<script setup>
import NavigationMenu from "./components/NavigationMenu.vue";
import AppContent from "./components/AppContent.vue";
console.log("Illusionna has started!");
</script>

<template>
  <NavigationMenu></NavigationMenu>
  <AppContent class="app-content"></AppContent>
</template>

<style scoped>
.app-content {
  padding: 25px;
}
</style>