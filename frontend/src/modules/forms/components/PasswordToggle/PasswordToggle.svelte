<script lang="ts">
  import IconEyeOpen from 'icons/eye-16.svg';
  import IconEyeClosed from 'icons/eye-closed-16.svg';

  export let activeType = 'password';
</script>

<button type="button" class="u-button-reset password-toggle" on:click>
  {#if activeType === 'password'}
    <IconEyeOpen />
  {:else}
    <IconEyeClosed />
  {/if}
</button>

<style>
  .password-toggle {
    color: theme(--color-text-2);
    padding-top: 4px;
    transition: color 0.18s var(--transition-easing-cubic);
  }

  .password-toggle:hover,
  .password-toggle:active {
    color: theme(--color-text-3);
  }
</style>
