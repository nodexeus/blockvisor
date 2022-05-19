<script lang="ts">
  import IconCaret from 'icons/caret-12.svg';

  export let title: string;
  export let content: string;

  let contentVisible: boolean = false;

  function toggleContent() {
    contentVisible = !contentVisible;
  }
</script>

<article class={`${contentVisible ? 'faq-item faq-item__active' : 'faq-item'}`}>
  <h2 class="faq-item__title">
    {title}
    <button class="u-button-reset faq-item__action" on:click={toggleContent}
      ><IconCaret /></button
    >
  </h2>
  {#if contentVisible}
    <p class="faq-item__content">{content}</p>
  {/if}
</article>

<style>
  .faq-item {
    margin-bottom: 24px;
    padding-bottom: 24px;
    border-bottom: 1px solid theme(--color-text-5-o10);
  }

  .faq-item__title {
    display: flex;
    justify-content: space-between;
    align-items: center;
    transition: all 0.15s ease-out;
    margin-bottom: 12px;
  }

  .faq-item__active .faq-item__action {
    transform: rotate(0deg);
  }

  .faq-item__action {
    color: theme(--color-text-5-o10);
    padding: 6px;
    border: 1px solid theme(--color-text-5-o10);
    border-radius: 4px;
    transition: all 0.1s ease-out;
    transform: rotate(180deg);
  }

  .faq-item__content {
    color: theme(--color-text-4);
    @mixin font small;
  }
</style>
