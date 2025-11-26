<template>
  <div class="modern-app">
    <header class="app-header">
      <div class="header-left">
        <div class="actions-group">
          <button class="btn-primary" @click="handleSelectFolder">
            <span class="icon">📂</span>
            <div v-if="folderPath">
              {{ folderPath }}
            </div>
            <div v-else>Select Folder</div>
          </button>
          <button
              v-if="folderPath"
              class="btn-secondary"
              @click="handleBuildIndex"
              :disabled="isBuildingIndex"
          >
            {{ isBuildingIndex ? 'Loading...' : 'Refresh' }}
          </button>
        </div>
      </div>

      <h2 class="app-logo" v-if="!isOperationMode">osu! mappack creator</h2>
      <div class="operation-summary" v-if="isOperationMode">
              <label class="add-delete-toggle">
                <input type="checkbox" v-model="includeDelete">
                <span>Add delete</span>
              </label>
              <button
                  class="btn-primary"
                  @click="performSelectionOperation"
                  :disabled="!canRunOperation || isRunningOperation"
              >
                {{ isRunningOperation ? 'Processing...' : 'Create Pack' }}
              </button>
            </div>

      <div class="header-right">
        <div class="status-pills" v-if="folderPath">
          <div class="pill" v-if="hasIndex">
            <span class="pill-label">Beatmap Count</span>
            <span class="pill-value">{{ allBeatmaps.length }}</span>
          </div>
          <div class="pill" v-if="lastDurationMs">
            <span class="pill-label">Time</span>
            <span class="pill-value">{{ durationText }}s</span>
          </div>
        </div>
        <a
            class="github-link"
            href="https://github.com/1skyyks1/osu-mappack-creator-v2"
            target="_blank"
            rel="noreferrer"
        >
          <svg class="github-icon" viewBox="0 0 24 24" aria-hidden="true">
            <path
                d="M12 .5a11.5 11.5 0 0 0-3.64 22.41c.58.11.79-.25.79-.56s0-1 0-2c-3.22.7-3.9-1.55-3.9-1.55a3.07 3.07 0 0 0-1.28-1.7c-1-.71.08-.7.08-.7a2.43 2.43 0 0 1 1.77 1.19 2.47 2.47 0 0 0 3.37 1 2.47 2.47 0 0 1 .74-1.55c-2.57-.29-5.27-1.29-5.27-5.73A4.51 4.51 0 0 1 5.2 8.19a4.2 4.2 0 0 1 .11-3.1s1-.33 3.3 1.26a11.31 11.31 0 0 1 6 0c2.27-1.59 3.29-1.26 3.29-1.26a4.2 4.2 0 0 1 .12 3.1 4.51 4.51 0 0 1 1.2 3.13c0 4.45-2.71 5.43-5.29 5.72a2.78 2.78 0 0 1 .79 2.16c0 1.56 0 2.82 0 3.2s.21.67.8.55A11.5 11.5 0 0 0 12 .5Z"
            />
          </svg>
        </a>
      </div>
    </header>

    <div class="app-body">
      <section class="main-section">
        <template v-if="!isOperationMode">
          <div class="search-container">
            <div class="search-wrapper">
              <span class="search-icon">🔍</span>
              <input
                  class="search-input"
                  type="text"
                  placeholder="Search Beatmap File Name..."
                  v-model="searchKeyword"
                  :disabled="!folderPath || isLoading"
              />
              <button
                  class="btn-text"
                  @click="clearSearchState"
                  v-if="trimmedKeyword || searchResults.length"
              >
                ✕ Clear
              </button>
            </div>
            <p v-if="isSearchActive && isSearching" class="search-hint">Searching...</p>
          </div>

          <div class="list-container scrollable">
            <div class="file-grid" v-if="displayedFiles.length">
              <div
                  v-for="file in displayedFiles"
                  :key="file.id"
                  @click="toggleSelection(file)"
                  class="file-card"
                  :class="{ 'is-selected': isBeatmapSelected(file) }"
              >
                <div class="file-icon">🎵</div>
                <div class="file-content">
                  <span class="file-id">{{ file.id }}</span>
                </div>
                <div class="selection-badge" :class="{ 'is-active': isBeatmapSelected(file) }">
                  <span v-if="isBeatmapSelected(file)">✓</span>
                  <span v-else>+</span>
                </div>
              </div>
            </div>

            <div
                v-else-if="folderPath && !isLoading && !isSearching"
                class="empty-state"
            >
              <div class="empty-icon">📭</div>
              <p>{{ isSearchActive ? 'No matching beatmaps found' : 'No data available' }}</p>
            </div>

            <div v-if="activeTotalPages > 0" class="pagination-bar">
              <button class="page-btn" @click="goPrev" :disabled="isLoading || currentPage === 0">
                ←
              </button>
              <span class="page-info">Page <b>{{ currentPage + 1 }}</b> of {{ activeTotalPages }}</span>
              <button class="page-btn" @click="goNext" :disabled="isLoading || currentPage >= activeTotalPages - 1">
                →
              </button>
            </div>
          </div>
        </template>

        <div v-else class="operation-panel">
          <div class="operation-body">

            <div class="pack-form">
              <label>Pack Title</label>
              <input class="input" type="text" v-model="packTitle" placeholder="Pack Title">
              <label>Pack Artist</label>
              <input class="input" type="text" v-model="packArtist" placeholder="Pack Artist">
              <label>Pack Creator</label>
              <input class="input" type="text" v-model="packCreator" placeholder="Pack Creator">
            </div>

            <div class="output-folder-row">
              <div class="output-folder-info">
                <label>Output Folder</label>
              </div>
              <span class="output-folder-path">
                  {{ outputFolder || 'Default (AppData/Local/Temp)' }}
                </span>
              <button class="btn-secondary btn-compact" type="button" @click="handleSelectOutputFolder">
                Choose
              </button>
            </div>

            <div class="operation-list scrollable">
              <div v-if="isFetchingDetails" class="operation-loader">Loading beatmap details...</div>
              <div
                  v-for="beatmap in selectedBeatmaps"
                  :key="'op-' + beatmap.path"
                  class="operation-item"
              >
                <div class="operation-item-header">
                  <div class="operation-item-info">
                    <span class="operation-title">{{ beatmap.displayName || beatmap.id }}</span>
                  </div>
                  <button class="collapse-btn" type="button" @click.stop="toggleBeatmapEditor(beatmap.path)">
                    {{ expandedBeatmapPath === beatmap.path ? '▼' : '▶' }}
                  </button>
                </div>
                <div
                    v-if="expandedBeatmapPath === beatmap.path"
                    class="operation-fields"
                >
                  <label>Version</label>
                  <input
                      class="input"
                      type="text"
                      v-model="beatmap.newVersion"
                      :placeholder="beatmap.displayName || beatmap.id"
                  >
                  <div class="operation-field-row is-inline">
                    <div class="operation-field">
                      <label>HP</label>
                      <input
                          class="input"
                          type="number"
                          step="0.1"
                          min="0"
                          max="10"
                          v-model.number="beatmap.hpDrainRate"
                          :placeholder="beatmap.metadata?.hp_drain_rate ?? DEFAULT_HP_DRAIN_RATE"
                      >
                    </div>
                    <div class="operation-field">
                      <label>OD</label>
                      <input
                          class="input"
                          type="number"
                          step="0.1"
                          min="0"
                          max="10"
                          v-model.number="beatmap.overallDifficulty"
                          :placeholder="beatmap.metadata?.overall_difficulty ?? DEFAULT_OVERALL_DIFFICULTY"
                      >
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <p v-if="operationMessage" class="operation-message">{{ operationMessage }}</p>
          </div>
        </div>
      </section>

      <aside class="selection-sidebar">
        <div class="selection-card">
          <div class="selection-header">
            <h3>Selected ({{ selectedBeatmaps.length }})</h3>
            <div class="selection-actions">
              <button
                  v-if="selectedBeatmaps.length"
                  class="btn-text"
                  @click="clearSelections"
              >
                Clear All
              </button>
              <button
                  v-if="!isOperationMode && selectedBeatmaps.length"
                  class="btn-primary btn-compact"
                  @click="enterOperationMode"
              >
                Next
              </button>
              <button
                  v-if="isOperationMode"
                  class="btn-secondary btn-compact"
                  @click="exitOperationMode"
              >
                Back
              </button>
            </div>
          </div>

          <div v-if="!selectedBeatmaps.length" class="selection-empty">
            <p>Currently no beatmaps selected.</p>
          </div>

          <div v-else class="selection-list scrollable">
            <div
                v-for="beatmap in selectedBeatmaps"
                :key="beatmap.path"
                class="selection-item"
            >
              <div class="selection-info">
                <span class="selection-title">{{ beatmap.displayName || beatmap.id }}</span>
                <span class="selection-path">{{ beatmap.path }}</span>
              </div>
              <button class="remove-btn" @click="removeSelection(beatmap)">✕</button>
            </div>
          </div>
        </div>
      </aside>

    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { logError } from './utils/logger'

const folderPath = ref('')
const files = ref([])
const allBeatmaps = ref([])
const selectedBeatmaps = ref([])
const isOperationMode = ref(false)
const pageSize = 13
const currentPage = ref(0)
const totalPages = ref(0)
const isLoading = ref(false)
const isBuildingIndex = ref(false)
const searchKeyword = ref('')
const searchResults = ref([])
const isSearching = ref(false)
const indexFilePath = ref('')
const progressFolder = ref('')
const scannedCount = ref(0)
const lastDurationMs = ref(0)
const packTitle = ref('')
const packArtist = ref('Various Artist')
const packCreator = ref('')
const includeDelete = ref(false)
const outputFolder = ref('')
const isFetchingDetails = ref(false)
const isRunningOperation = ref(false)
const operationMessage = ref('')
const expandedBeatmapPath = ref('')
let searchTimer = null
let progressUnlisten = null

const trimmedKeyword = computed(() => searchKeyword.value.trim())
const isSearchActive = computed(() => !!trimmedKeyword.value)
const hasIndex = computed(() => allBeatmaps.value.length > 0)
const searchTotalPages = computed(() => {
  if (!isSearchActive.value) return 0
  return Math.ceil(searchResults.value.length / pageSize) || 0
})
const activeTotalPages = computed(() => (isSearchActive.value ? searchTotalPages.value : totalPages.value))
const pagedSearchResults = computed(() => {
  if (!isSearchActive.value) return []
  const start = currentPage.value * pageSize
  return searchResults.value.slice(start, start + pageSize)
})
const displayedFiles = computed(() => (isSearchActive.value ? pagedSearchResults.value : files.value))
const shouldShowProgress = computed(() => isBuildingIndex.value || scannedCount.value > 0)
const durationText = computed(() => {
  if (!lastDurationMs.value) return ''
  return (lastDurationMs.value / 1000).toFixed(2)
})

const canRunOperation = computed(() => {
  return (
      selectedBeatmaps.value.length > 0 &&
      !!packTitle.value.trim() &&
      !!packArtist.value.trim() &&
      !!packCreator.value.trim()
  )
})

const resetPackState = () => {
  packTitle.value = ''
  packArtist.value = 'Various Artist'
  packCreator.value = ''
  includeDelete.value = false
  operationMessage.value = ''
  isRunningOperation.value = false
  expandedBeatmapPath.value = ''
}

const formatBeatmapLabel = (metadata, fallback) => {
  if (!metadata) return fallback
  const artist = metadata.artist?.trim()
  const title = metadata.title?.trim()
  const creator = metadata.creator?.trim()
  if (!artist && !title && !creator) return fallback
  return `${artist || 'Unknown'} - ${title || 'Untitled'} [${creator || 'Unknown'}]`
}

const formatBeatmapVersionLabel = (metadata, fallback) => {
  if (!metadata) return fallback
  const artist = metadata.artist?.trim() || 'Unknown'
  const title = metadata.title?.trim() || 'Untitled'
  const creator = metadata.creator?.trim() || 'Unknown'
  const version = metadata.version?.trim() || 'Unnamed'
  return `${artist} - ${title} [${creator}] (${version})`
}

const DEFAULT_HP_DRAIN_RATE = 5.0
const DEFAULT_OVERALL_DIFFICULTY = 5.0

const toNumberOr = (value, fallback) => {
  const parsed = Number(value)
  return Number.isFinite(parsed) ? parsed : fallback
}

const mergeBeatmapDetails = (details) => {
  if (!Array.isArray(details) || !details.length) return
  const detailMap = new Map(details.map((item) => [item.path, item]))
  selectedBeatmaps.value = selectedBeatmaps.value.map((beatmap) => {
    const detail = detailMap.get(beatmap.path)
    if (!detail) return beatmap
    const label = formatBeatmapLabel(detail.metadata, beatmap.id)
    const versionLabel = formatBeatmapVersionLabel(detail.metadata, label)
    const metadataHp = toNumberOr(detail.metadata.hp_drain_rate, DEFAULT_HP_DRAIN_RATE)
    const metadataOd = toNumberOr(detail.metadata.overall_difficulty, DEFAULT_OVERALL_DIFFICULTY)
    const hpDrainRate = (beatmap.hpDrainRate === null || beatmap.hpDrainRate === undefined)
      ? metadataHp
      : toNumberOr(beatmap.hpDrainRate, metadataHp)
    const overallDifficulty = (beatmap.overallDifficulty === null || beatmap.overallDifficulty === undefined)
      ? metadataOd
      : toNumberOr(beatmap.overallDifficulty, metadataOd)
    return {
      ...beatmap,
      metadata: detail.metadata,
      displayName: label,
      newVersion: beatmap.newVersion || versionLabel,
      hpDrainRate,
      overallDifficulty
    }
  })
  ensureExpandedBeatmap()
}

const ensureExpandedBeatmap = () => {
  if (!selectedBeatmaps.value.length) {
    expandedBeatmapPath.value = ''
    return
  }
  if (!expandedBeatmapPath.value || !selectedBeatmaps.value.some((item) => item.path === expandedBeatmapPath.value)) {
    expandedBeatmapPath.value = selectedBeatmaps.value[0].path
  }
}

const fetchBeatmapDetails = async (paths) => {
  if (!Array.isArray(paths) || !paths.length) return
  const uniquePaths = Array.from(new Set(paths))
  isFetchingDetails.value = true
  try {
    const result = await invoke('load_beatmap_details', { paths: uniquePaths })
    mergeBeatmapDetails(Array.isArray(result) ? result : [])
  } catch (err) {
    await logError('Failed to load beatmap details', err)
    operationMessage.value = err?.message || 'Failed to load beatmap details'
  } finally {
    isFetchingDetails.value = false
  }
}

const clearSelections = () => {
  selectedBeatmaps.value = []
  isOperationMode.value = false
  resetPackState()
}

const isBeatmapSelected = (beatmap) => {
  if (!beatmap?.path) return false
  return selectedBeatmaps.value.some((item) => item.path === beatmap.path)
}

const addBeatmapToSelection = (beatmap) => {
  if (!beatmap?.path || isBeatmapSelected(beatmap)) return
  let hpDrainRate = null
  let overallDifficulty = null
  
  if (beatmap.hpDrainRate !== null && beatmap.hpDrainRate !== undefined) {
    hpDrainRate = beatmap.hpDrainRate
  } else if (beatmap.metadata?.hp_drain_rate !== undefined) {
    hpDrainRate = toNumberOr(beatmap.metadata.hp_drain_rate, DEFAULT_HP_DRAIN_RATE)
  }
  
  if (beatmap.overallDifficulty !== null && beatmap.overallDifficulty !== undefined) {
    overallDifficulty = beatmap.overallDifficulty
  } else if (beatmap.metadata?.overall_difficulty !== undefined) {
    overallDifficulty = toNumberOr(beatmap.metadata.overall_difficulty, DEFAULT_OVERALL_DIFFICULTY)
  }
  
  const entry = {
    id: beatmap.id,
    path: beatmap.path,
    metadata: beatmap.metadata ?? null,
    newVersion: beatmap.newVersion ?? '',
    hpDrainRate,
    overallDifficulty,
    displayName: beatmap.displayName ?? beatmap.id
  }
  selectedBeatmaps.value = [...selectedBeatmaps.value, entry]
  ensureExpandedBeatmap()
}

const removeSelection = (beatmap) => {
  if (!beatmap?.path) return
  selectedBeatmaps.value = selectedBeatmaps.value.filter((item) => item.path !== beatmap.path)
  ensureExpandedBeatmap()
}

const toggleSelection = (beatmap) => {
  if (!beatmap?.path) return
  if (isBeatmapSelected(beatmap)) {
    removeSelection(beatmap)
    return
  }
  addBeatmapToSelection(beatmap)
}

const toggleBeatmapEditor = (path) => {
  if (!path) return
  expandedBeatmapPath.value = expandedBeatmapPath.value === path ? '' : path
}

const enterOperationMode = async () => {
  if (!selectedBeatmaps.value.length) return
  operationMessage.value = ''
  isOperationMode.value = true
  const targetPaths = selectedBeatmaps.value.map((item) => item.path)
  await fetchBeatmapDetails(targetPaths)
}

const exitOperationMode = () => {
  isOperationMode.value = false
  resetPackState()
}

const clearSearchState = () => {
  if (searchTimer) {
    clearTimeout(searchTimer)
    searchTimer = null
  }
  searchKeyword.value = ''
  searchResults.value = []
  isSearching.value = false
  currentPage.value = 0
}

const applyIndexPage = (pageNumber = 0) => {
  const total = allBeatmaps.value.length
  totalPages.value = total ? Math.ceil(total / pageSize) : 0
  if (totalPages.value === 0) {
    files.value = []
    currentPage.value = 0
    return
  }
  const safePage = Math.min(Math.max(pageNumber, 0), totalPages.value - 1)
  const start = safePage * pageSize
  files.value = allBeatmaps.value.slice(start, start + pageSize)
  currentPage.value = safePage
}

const fetchPage = async (pageNumber = 0) => {
  if (!folderPath.value) return

  if (hasIndex.value) {
    applyIndexPage(pageNumber)
    return
  }

  if (isLoading.value) return
  isLoading.value = true

  try {
    const result = await invoke('load_osu_files_page', {
      osuPath: folderPath.value,
      page: pageNumber,
      pageSize: pageSize
    })

    const beatmaps = Array.isArray(result?.beatmaps) ? result.beatmaps : []
    files.value = beatmaps
    totalPages.value = result?.total_pages ?? 0
    currentPage.value = pageNumber
    clearSearchState()
  } catch (err) {
    await logError('Failed to load beatmap page', err)
  } finally {
    isLoading.value = false
  }
}

const handleSelectFolder = async () => {
  try {
    const path = await invoke('select_osu_path')
    if (path) {
      folderPath.value = path
      files.value = []
      allBeatmaps.value = []
      clearSelections()
      isOperationMode.value = false
      totalPages.value = 0
      currentPage.value = 0
      clearSearchState()
      indexFilePath.value = ''
      progressFolder.value = ''
      scannedCount.value = 0
      lastDurationMs.value = 0
      const built = await buildIndex()
      if (!built) {
        await fetchPage(0)
      }
    }
  } catch (err) {
    await logError('Failed to select osu folder', err)
  }
}

const handleSelectOutputFolder = async () => {
  try {
    const path = await invoke('select_output_folder')
    if (path) {
      outputFolder.value = path
    }
  } catch (err) {
    await logError('Failed to select output folder', err)
  }
}

const changePage = (pageNumber) => {
  const total = activeTotalPages.value
  if (total === 0 || pageNumber < 0 || pageNumber >= total || pageNumber === currentPage.value) {
    return
  }

  if (isSearchActive.value) {
    currentPage.value = pageNumber
    return
  }

  fetchPage(pageNumber)
}

const goPrev = () => {
  changePage(currentPage.value - 1)
}

const goNext = () => {
  changePage(currentPage.value + 1)
}

const runSearch = async (keyword) => {
  if (!folderPath.value) return

  if (hasIndex.value) {
    isSearching.value = true
    const lower = keyword.toLowerCase()
    searchResults.value = allBeatmaps.value.filter((beatmap) =>
        beatmap.id.toLowerCase().includes(lower)
    )
    isSearching.value = false
    currentPage.value = 0
    return
  }

  isSearching.value = true
  try {
    const result = await invoke('search_beatmaps_by_id', {
      osuPath: folderPath.value,
      keyword
    })

    searchResults.value = Array.isArray(result) ? result : []
    currentPage.value = 0
  } catch (err) {
    await logError('Failed to search beatmaps', err)
  } finally {
    isSearching.value = false
  }
}

watch(trimmedKeyword, (value) => {
  if (!folderPath.value) return
  if (searchTimer) {
    clearTimeout(searchTimer)
    searchTimer = null
  }

  if (!value) {
    searchResults.value = []
    isSearching.value = false
    currentPage.value = 0
    return
  }

  searchTimer = window.setTimeout(() => {
    runSearch(value)
  }, 300)
})

const triggerSearch = () => {
  if (!trimmedKeyword.value) {
    clearSearchState()
    return
  }

  if (searchTimer) {
    clearTimeout(searchTimer)
    searchTimer = null
  }
  runSearch(trimmedKeyword.value)
}

watch(selectedBeatmaps, (value) => {
  if (!value.length) {
    isOperationMode.value = false
    resetPackState()
    return
  }
  ensureExpandedBeatmap()
  if (isOperationMode.value) {
    const pending = value.filter((item) => !item.metadata).map((item) => item.path)
    if (pending.length) {
      fetchBeatmapDetails(pending)
    }
  }
})

const performSelectionOperation = async () => {
  if (!selectedBeatmaps.value.length || isRunningOperation.value) return
  if (!packTitle.value.trim() || !packArtist.value.trim() || !packCreator.value.trim()) {
    operationMessage.value = 'Please input pack title / artist / creator'
    return
  }

  const pending = selectedBeatmaps.value.filter((item) => !item.metadata).map((item) => item.path)
  if (pending.length) {
    await fetchBeatmapDetails(pending)
  }

  const payload = selectedBeatmaps.value.map((beatmap) => {
    const fallbackMetadata = beatmap.metadata || {
      hp_drain_rate: DEFAULT_HP_DRAIN_RATE,
      overall_difficulty: DEFAULT_OVERALL_DIFFICULTY,
      artist: '',
      title: '',
      creator: '',
      version: ''
    }
    const defaultHp = toNumberOr(fallbackMetadata.hp_drain_rate, DEFAULT_HP_DRAIN_RATE)
    const defaultOd = toNumberOr(fallbackMetadata.overall_difficulty, DEFAULT_OVERALL_DIFFICULTY)
    const versionValue = beatmap.newVersion?.trim() || formatBeatmapVersionLabel(fallbackMetadata, beatmap.id)
    return {
      path: beatmap.path,
      new_version: versionValue,
      hp_drain_rate: toNumberOr(beatmap.hpDrainRate, defaultHp),
      overall_difficulty: toNumberOr(beatmap.overallDifficulty, defaultOd)
    }
  })

  isRunningOperation.value = true
  operationMessage.value = ''
  try {
    const trimmedTitle = packTitle.value.trim()
    const trimmedArtist = packArtist.value.trim()
    const trimmedCreator = packCreator.value.trim()
    const selectedOutput = outputFolder.value?.trim() || null
    const includeDeleteFiles = includeDelete.value
    const result = await invoke('create_pack', {
      pack_title: trimmedTitle,
      pack_artist: trimmedArtist,
      pack_creator: trimmedCreator,
      packTitle: trimmedTitle,
      packArtist: trimmedArtist,
      packCreator: trimmedCreator,
      output_dir: selectedOutput,
      outputDir: selectedOutput,
      include_extra_files: includeDeleteFiles,
      includeExtraFiles: includeDeleteFiles,
      beatmaps: payload
    })
    operationMessage.value = `Pack created: ${result?.osz_path || 'Unknown Path'}`
  } catch (err) {
    await logError('Failed to create pack', err)
    operationMessage.value = err?.message || 'Failed to create pack'
  } finally {
    isRunningOperation.value = false
  }
}

const buildIndex = async () => {
  if (!folderPath.value || isBuildingIndex.value) return false

  isBuildingIndex.value = true
  progressFolder.value = 'Preparing...'
  scannedCount.value = 0
  lastDurationMs.value = 0
  try {
    const result = await invoke('build_beatmap_index', {
      osuPath: folderPath.value
    })

    const beatmaps = Array.isArray(result?.beatmaps) ? result.beatmaps : []
    allBeatmaps.value = beatmaps
    indexFilePath.value = result?.index_path ?? ''
    lastDurationMs.value = result?.duration_ms ?? 0
    clearSearchState()
    applyIndexPage(0)
    return true
  } catch (err) {
    await logError('Failed to build beatmap index', err)
    return false
  } finally {
    isBuildingIndex.value = false
  }
}

const handleBuildIndex = async () => {
  await buildIndex()
}

const setupProgressListener = async () => {
  if (progressUnlisten) return
  progressUnlisten = await listen('index-progress', (event) => {
    const payload = event.payload || {}
    progressFolder.value = payload.folder || ''
    scannedCount.value = payload.scanned ?? 0
  })
}

onMounted(() => {
  setupProgressListener()
})

onUnmounted(() => {
  if (progressUnlisten) {
    progressUnlisten()
    progressUnlisten = null
  }
})
</script>

<style>
* {
  box-sizing: border-box;
}

:root {
  --c-accent: #0070f3;
  --c-accent-hover: #0761d1;
  --c-bg-app: #fafafa;
  --c-bg-panel: #ffffff;
  --c-text-primary: #111111;
  --c-text-secondary: #666666;
  --c-border: #d5d5d5;
  --c-hover: #f5f5f5;
  --radius: 6px;
  --font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
}

.modern-app {
  font-family: var(--font-sans);
  font-size: 14px;
  color: var(--c-text-primary);
  background-color: var(--c-bg-app);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.app-header {
  background: var(--c-bg-panel);
  border-bottom: none;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 10px;
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 20px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.app-logo {
  font-weight: 700;
  font-size: 16px;
  margin: 0;
  letter-spacing: -0.02em;
}

.actions-group {
  display: flex;
  gap: 10px;
}

.github-link {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  height: 34px;
  width: 34px;
  border-radius: 50%;
  border: 1px solid #ddd;
  font-size: 12px;
  text-decoration: none;
  color: var(--c-text-primary);
  transition: all 0.15s ease;
  white-space: nowrap;
}
.github-link:hover {
  border-color: var(--c-text-primary);
  color: #000;
  background: #fff;
}

.github-icon {
  width: 20px;
  height: 20px;
  fill: currentColor;
}

button {
  font-family: inherit;
  font-size: 13px;
  font-weight: 500;
  padding: 0 16px;
  height: 32px;
  border-radius: var(--radius);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  transition: all 0.15s ease;
  white-space: nowrap;
}

.btn-compact {
  height: 28px;
  padding: 0 10px;
  font-size: 12px;
}

.btn-primary {
  background-color: #000;
  color: #fff;
  border: 1px solid #000;
}
.btn-primary:hover {
  background-color: #333;
  border-color: #333;
  transform: translateY(-1px);
}
.btn-primary:active { transform: translateY(0); }

.btn-secondary {
  background-color: #fff;
  color: var(--c-text-primary);
  border: 1px solid #ddd;
}
.btn-secondary:hover:not(:disabled) {
  border-color: var(--c-text-primary);
  color: #000;
}
.btn-secondary:disabled {
  background-color: #f5f5f5;
  color: #999;
  border-color: #eee;
  cursor: not-allowed;
}

.status-pills {
  display: flex;
  gap: 12px;
}
.pill {
  font-size: 12px;
  color: var(--c-text-secondary);
  background: var(--c-bg-app);
  padding: 4px 8px;
  border-radius: 4px;
}
.pill-value {
  color: var(--c-text-primary);
  font-weight: 600;
  margin-left: 4px;
}

.app-body {
  display: flex;
  flex: 1;
  overflow: hidden;
  gap: 10px;
}

.main-section {
  height: calc(100vh - 70px);
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--c-bg-panel);
  border-right: none;
  min-width: 0;
  transition: all 0.2s ease;
}

.search-container {
  padding: 6px 10px;
  border-bottom: none;
}

.search-wrapper {
  display: flex;
  align-items: center;
  background: var(--c-bg-app);
  border: 1px solid var(--c-border);
  border-radius: var(--radius);
  padding: 2px;
  transition: border 0.2s;
}

.search-wrapper:focus-within {
  border-color: var(--c-text-secondary);
  background: #fff;
}

.search-icon {
  padding: 0 4px;
  font-size: 12px;
  color: #999;
}

.search-input {
  flex: 1;
  border: none;
  background: transparent;
  height: 26px;
  font-size: 12px;
  outline: none;
  color: var(--c-text-primary);
}

.btn-icon {
  height: 24px;
  padding: 0 12px;
  background: var(--c-text-primary);
  color: #fff;
  border: none;
  border-radius: 4px;
  font-size: 12px;
}
.btn-icon:disabled { background: #ccc; }

.btn-text {
  border: none;
  background: none;
  color: #999;
  font-size: 12px;
  padding: 0 10px;
}
.btn-text:hover { color: #f00; }
.search-hint { margin-top: 8px; font-size: 12px; color: var(--c-text-secondary); }

.list-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 0 10px;
}

.scrollable {
  overflow-y: auto;
}

.path-breadcrumb {
  padding: 10px 12px;
  font-size: 12px;
  color: var(--c-text-secondary);
  background: var(--c-bg-app);
  border-bottom: none;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-grid {
  overflow-y: auto;
  padding: 0;
}

.file-card {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 6px 10px;
  border-bottom: none;
  cursor: pointer;
  background: #fff;
  transition: background 0.1s, border 0.1s;
  border-left: 3px solid transparent;
}

.file-card:hover {
  background-color: var(--c-hover);
}

.file-card.is-selected {
  border-left-color: var(--c-accent);
  background-color: #f0f7ff;
}

.file-icon {
  margin-right: 6px;
  font-size: 14px;
  opacity: 0.6;
}

.file-content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.file-id {
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-path {
  font-size: 10px;
  color: var(--c-text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.selection-badge {
  width: 16px;
  height: 16px;
  border-radius: var(--radius);
  border: 1px solid var(--c-border);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  color: var(--c-text-secondary);
  transition: all 0.15s ease;
  line-height: 1;
}

.selection-badge span {
  position: relative;
  top: -1px;
}

.selection-badge.is-active {
  background-color: var(--c-accent);
  border-color: var(--c-accent);
  color: #fff;
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #999;
}
.empty-icon { font-size: 32px; margin-bottom: 10px; opacity: 0.3; }

.pagination-bar {
  padding: 10px 10px 0;
  border-top: none;
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 15px;
  background: #fff;
}
.page-btn {
  background: transparent;
  border: none;
  color: var(--c-text-primary);
  width: 32px;
  padding: 0;
}
.page-btn:hover:not(:disabled) { border-color: #999; }

.selection-sidebar {
  width: 320px;
  height: calc(100vh - 70px);
  background: var(--c-bg-app);
  padding: 10px 10px 10px 4px;
  border-left: none;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  flex-shrink: 0;
}

.selection-card {
  background: transparent;
  border: none;
  padding: 0;
  box-shadow: none;
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}

.selection-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 10px;
  margin-bottom: 16px;
}

.selection-header h3 {
  font-size: 16px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: #373737;
  margin: 0;
  font-weight: 700;
}

.selection-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.selection-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  overflow-y: auto;
  padding-right: 6px;
  flex: 1;
  min-height: 0;
}

.scrollable::-webkit-scrollbar {
  width: 6px;
}
.scrollable::-webkit-scrollbar-track {
  background: transparent;
}
.scrollable::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.25);
  border-radius: 999px;
}
.scrollable::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 0, 0, 0.4);
}

.scrollable {
  scrollbar-width: thin;
  scrollbar-color: rgba(0, 0, 0, 0.25) transparent;
}

.selection-empty {
  margin-top: 40px;
  text-align: center;
  color: #bbb;
  font-size: 13px;
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.selection-item {
  display: flex;
  gap: 12px;
  padding: 6px 8px;
  border-radius: 6px;
  background: #fff;
  border: 1px solid var(--c-border);
}

.selection-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.selection-title {
  font-size: 13px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.selection-path {
  font-size: 9px;
  color: var(--c-text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.remove-btn {
  border: none;
  background: transparent;
  color: #999;
  font-size: 14px;
  cursor: pointer;
  align-self: flex-start;
  padding: 4px; 
}

.remove-btn:hover {
  color: #f00;
}

.operation-panel {
  width: 100%;
  flex: 1;
  background: var(--c-bg-panel);
  padding: 6px 10px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  overflow: hidden;
  min-height: 0;
}

.operation-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
  overflow: hidden;
  min-height: 0;
}

.operation-summary {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.summary-content {
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.summary-count {
  font-size: 32px;
  font-weight: 700;
}

.summary-label {
  font-size: 12px;
  color: var(--c-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.pack-form {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  gap: 10px;
  align-items: center;
}

.pack-form label {
  display: flex;
  align-items: center;
  height: 100%;
}

.output-folder-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  background: #fff;
  border-radius: var(--radius);
}

.output-folder-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
}

.output-folder-info label {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--c-text-secondary);
}

.output-folder-path {
  font-size: 12px;
  color: var(--c-text-primary);
  word-break: break-all;
}

.add-delete-toggle {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--c-text-primary);
  user-select: none;
}

.add-delete-toggle input {
  width: 14px;
  height: 14px;
}

.input {
  border: 1px solid var(--c-border);
  border-radius: var(--radius);
  padding: 6px 8px;
  font-size: 12px;
  width: 100%;
  background: #fff;
}

.operation-list {
  flex: 1;
  overflow-y: auto;
  border: 1px solid var(--c-border);
  border-radius: var(--radius);
  padding: 4px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.operation-loader {
  font-size: 12px;
  color: var(--c-text-secondary);
  padding: 8px;
  border: 1px dashed var(--c-border);
  border-radius: var(--radius);
}

.operation-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 0 4px 4px;
  border-bottom: 1px solid #eee;
}

.operation-item:last-child {
  border-bottom: none;
}

.operation-item-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 10px;
  cursor: pointer;
}

.operation-item-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.operation-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--c-text-primary);
}

.operation-path {
  font-size: 10px;
  color: var(--c-text-secondary);
  word-break: break-all;
}

.operation-fields {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.operation-fields label,
.pack-form label {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--c-text-secondary);
}

.operation-field-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.operation-field-row.is-inline {
  flex-direction: row;
  gap: 12px;
}

.operation-field {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.collapse-btn {
  border: none;
  background: #fff;
  color: var(--c-text-primary);
  font-size: 11px;
  border-radius: var(--radius);
  padding: 4px 10px;
  cursor: pointer;
}

.collapse-btn:hover {
  border-color: var(--c-text-primary);
}

.operation-message {
  font-size: 12px;
  color: var(--c-text-primary);
  margin-top: 4px;
}

</style>