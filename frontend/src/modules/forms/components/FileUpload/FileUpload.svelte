<script lang="ts">
  import Dropzone from 'svelte-file-dropzone';
  import IconUpload from 'icons/upload-24.svg';
  import IconCheck from 'icons/checkmark-24.svg';
  import IconDelete from 'icons/close-12.svg';

  export let files = [];

  $: hasFiles = Boolean(files.length);
  $: isDragging = false;

  const handleDragStart = () => {
    isDragging = true;
  };
  const handleDragEnd = () => {
    isDragging = false;
  };

  $: containerClasses = [
    't-small file-upload__input',
    `file-upload__input--${hasFiles ? 'has-files' : 'no-files'}`,
    `file-upload__input--${isDragging ? 'dragging' : 'not-dragging'}`,
  ].join(' ');
</script>

<div class="file-upload">
  <Dropzone
    disabled={hasFiles}
    {containerClasses}
    multiple={false}
    disableDefaultStyles
    on:dragover={handleDragStart}
    on:dragleave={handleDragEnd}
    on:dropaccepted={handleDragEnd}
    on:droprejected={handleDragEnd}
    on:drop
    {...$$props}
  >
    {#if hasFiles}
      <IconCheck />
      <span class="file-upload__label">
        {files.map(({ name }) => name).join(', ')}
      </span>
      <button on:click class="u-button-reset file-upload__remove">
        <IconDelete />
      </button>
    {:else}
      <IconUpload />
      <slot>Select a file to upload</slot>
    {/if}
  </Dropzone>
</div>

<style>
  .file-upload {
    & :global(.file-upload__input) {
      position: relative;
      display: inline-flex;
      gap: 8px;
      align-items: center;
      padding: 15px 22px;
      border: 1px dashed theme(--color-border-2);
      border-radius: 4px;
      cursor: pointer;
    }

    &__label {
      max-width: 22ch;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    & :global(.file-upload__input--no-files) {
      &:hover,
      &:active {
        background-color: theme(--color-text-5-o3);
      }
    }

    & :global(.file-upload__input--no-files.file-upload__input--dragging) {
      background-color: theme(--color-text-5-o3);
    }

    & :global(.file-upload__input--has-files) {
      cursor: auto;
    }

    &__remove {
      padding-left: 6px;
      cursor: pointer;

      & :global(svg) {
        opacity: 1;
      }

      & :global(path) {
        fill: theme(--color-text-4);
        opacity: 1;
        transition: fill 0.18s var(--transition-easing-cubic);
      }

      &:hover,
      &:active {
        & :global(svg) {
          opacity: 1;
        }

        & :global(path) {
          fill: theme(--color-text-5);
          opacity: 1;
        }
      }
    }
  }
</style>
