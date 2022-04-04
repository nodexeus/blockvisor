<script lang="ts">
  import Input from 'modules/forms/components/Input/Input.svelte';
  import IconSearch from 'icons/search-12.svg';
  import { clickOutside } from 'utils';

  import { useForm } from 'svelte-use-form';
  import SearchResultsSidebar from '../SearchResultsSidebar/SearchResultsSidebar.svelte';
  import { afterUpdate } from 'svelte';

  const form = useForm();

  export let isOpen = false;

  export let handleOpen;
  export let handleClose;

  $: hasInput = Boolean($form?.search?.value.trim().length);

  let isSearchSidebarActive = false;

  const handleClickOutside = (e) => {
    handleClose();
    isSearchSidebarActive = false;
  };

  afterUpdate(() => {
    if (!hasInput) {
      isSearchSidebarActive = false;
    }

    if (isOpen && hasInput) {
      isSearchSidebarActive = true;
    }
  });

  $: formClass = isOpen ? 'search search--active' : 'search';
</script>

<form
  class={formClass}
  use:form
  use:clickOutside
  on:click_outside={handleClickOutside}
>
  <Input
    on:keyup={() => {
      isSearchSidebarActive = hasInput;
    }}
    on:focus={() => {
      isSearchSidebarActive = hasInput;
    }}
    size="small"
    placeholder="Start typing..."
    name="search"
    field={$form?.search}
    labelClass="visually-hidden"
  >
    <button
      on:click={handleOpen}
      type="button"
      class="search__button"
      slot="utilLeft"
    >
      <IconSearch />
    </button>
    <svelte:fragment slot="label">Search</svelte:fragment>
  </Input>
  <SearchResultsSidebar isActive={isSearchSidebarActive} />
</form>

<style>
  .search {
    &__button {
      background-color: transparent;
      color: theme(--color-border-2);
      border-width: 0;
      padding: 0;

      @media (--screen-medium-max) {
        cursor: pointer;
        padding: 8px 10px;
      }
    }

    @media (--screen-medium-max) {
      position: relative;
      min-height: 32px;

      & :global(input) {
        padding-left: 28px;
        overflow: hidden;
        transition: width 0.25s ease-out, padding 0.25s ease-out;
      }

      & :global(.input__wrapper) {
        position: absolute;
        right: 0;
      }

      & :global(.input-util) {
        left: 0;
      }

      &:not(.search--active) {
        & :global(input) {
          padding-left: 18px;
          width: 0;
        }
      }

      &--active {
        & :global(input) {
          width: 208px;
        }
      }
    }

    @media (--screen-smaller-custom-max) {
      &--active {
        & :global(input) {
          width: calc(45vw + 55px);
        }
      }
    }
  }
</style>
