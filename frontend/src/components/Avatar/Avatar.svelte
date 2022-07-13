<script lang="ts">
  import LazyImg from 'components/LazyImg/LazyImg.svelte';
  import type { AvatarProps } from './Avatar.svelte';

  export let src = '';
  export let fullName = 'John Doe';
  export let size: AvatarProps['size'] = 'small';

  const avatarClass = ['avatar', `avatar--${size}`].join(' ');

  const getInitials = () => {
    const [name = '', surname = ''] = fullName.split(' ');
    return `${name?.charAt(0) ?? ''}${surname?.charAt(0) ?? ''}`;
  };
</script>

<figure
  on:click
  on:focus
  on:blur
  on:keyup
  class={avatarClass}
  class:avatar--with-image={src}
  {...$$restProps}
>
  {#if src}
    <LazyImg
      width="40"
      height="40"
      class="avatar__image"
      {src}
      alt={`${fullName}'s avatar`}
    />
  {/if}
  <figcaption class="avatar__initials t-micro">
    {getInitials()}
  </figcaption>
</figure>

<style lang="postcss">
  .avatar {
    display: block;
    border-radius: 100%;
    aspect-ratio: 1;
    text-transform: uppercase;
    color: theme(--color-text-1);
    background-color: theme(--color-secondary);
    overflow: hidden;
    text-align: center;
    position: relative;
    isolation: isolate;

    &__initials {
      position: absolute;
      top: 50%;
      left: 50%;
      transform: translate3d(-50%, -50%, 0);
    }

    & :global(.avatar__image) {
      object-fit: cover;
      object-position: center;
      position: relative;
      z-index: var(--level-1);
      height: auto;
      color: transparent;
      height: 100%;
    }

    &--with-image {
      border: 1px solid theme(--color-border-4);
    }

    &--small {
      width: 24px;
      line-height: 1;
    }

    &--medium-small {
      width: 32px;
      line-height: 1;
    }

    &--medium {
      width: 40px;
      line-height: 1;
    }
  }
</style>
