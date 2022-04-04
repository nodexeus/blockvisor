<script lang="ts">
  export let className = '';
  const elementClasses = ['action-title', className].join(' ');
</script>

<header class={elementClasses}>
  <div class="container--large action-title__inner">
    <div class="action-title__main">
      <slot name="title" />
      <slot name="action" />
    </div>
    <slot />
  </div>
</header>

<style>
  .action-title {
    padding-top: 40px;
    padding-bottom: 40px;
    margin-bottom: 60px;
    background-color: theme(--color-text-5-o3);

    &__inner {
      gap: 20px;
      align-items: center;
      justify-content: space-between;
      flex-wrap: wrap;
      display: flex;
    }

    &__main {
      flex-wrap: wrap;
      gap: 20px;
      align-items: center;
      display: flex;
    }
  }
</style>
