<script setup>
const minimize = () => {
  window.window_operation.minimize();
};
const maximize = () => {
  window.window_operation.maximize();
}
const close = () => {
  window.window_operation.close();
}
</script>

<template>
  <nav class="navbar has-navbar-fixed-top is-black" role="navigation" aria-label="main navigation">
    <div class="navbar-brand">
      <RouterLink class="navbar-item" to="/">
        <img src="../../resources/title.png" alt="Illusionna Title">
      </RouterLink>
    </div>
    <div class="navbar-menu"></div>
    <div class="navbar-end">
      <i class="bi bi-caret-down-fill" @click="minimize"/>
      <i class="bi bi-arrows-fullscreen" @click="maximize"/>
      <i class="bi bi-x-lg" @click="close"/>
    </div>
  </nav>
</template>

<style scoped>
.navbar {
  user-select: none;
  -webkit-app-region: drag;
}
.navbar-end {
  display: flex;
  justify-content: space-between;
}
RouterLink {
  -webkit-app-region: no-drag;
}
i {
  font-size: 20px;
  padding: 10px;
  -webkit-app-region: no-drag;
}
</style>