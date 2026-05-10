import { getCurrentWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'

const dropZone = document.getElementById('drop-zone')
const dropText = document.getElementById('drop-text')
const dropSubtext = document.getElementById('drop-subtext')
const dropIcon = document.getElementById('drop-icon')
const status = document.getElementById('status')

const DEFAULT_TEXT = 'フォルダをドロップ'
const DEFAULT_SUBTEXT = 'ファイルを連番にリネームして filename.txt を作成します'

const ICON_DOWNLOAD = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
  <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5M16.5 12L12 16.5m0 0L7.5 12m4.5 4.5V3" />
</svg>`

const ICON_CHECK = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
  <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
</svg>`

const ICON_ERROR = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
  <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z" />
</svg>`

function setDropZoneState(state, message = '') {
  dropZone.className = state !== 'default' ? state : ''
  switch (state) {
    case 'dragging':
      dropIcon.innerHTML = ICON_DOWNLOAD
      dropText.textContent = 'ここにドロップ'
      dropSubtext.textContent = ''
      break
    case 'loading':
      dropIcon.innerHTML = ICON_DOWNLOAD
      dropText.textContent = '処理中...'
      dropSubtext.textContent = message
      break
    case 'success':
      dropIcon.innerHTML = ICON_CHECK
      dropText.textContent = '完了'
      dropSubtext.textContent = message
      break
    case 'error':
      dropIcon.innerHTML = ICON_ERROR
      dropText.textContent = 'エラー'
      dropSubtext.textContent = message
      break
    default:
      dropIcon.innerHTML = ICON_DOWNLOAD
      dropText.textContent = DEFAULT_TEXT
      dropSubtext.textContent = DEFAULT_SUBTEXT
  }
}

// Click anywhere to reset after success/error
dropZone.addEventListener('click', () => {
  const state = dropZone.className
  if (state === 'success' || state === 'error') {
    setDropZoneState('default')
  }
})

const appWindow = getCurrentWindow()

await appWindow.onDragDropEvent(async (event) => {
  const { type } = event.payload

  if (type === 'enter') {
    setDropZoneState('dragging')
  } else if (type === 'leave') {
    setDropZoneState('default')
  } else if (type === 'drop') {
    const { paths } = event.payload

    if (paths.length !== 1) {
      setDropZoneState('error', 'フォルダを1つだけドロップしてください')
      return
    }

    setDropZoneState('loading', paths[0])

    try {
      const result = await invoke('rename_files', { folderPath: paths[0] })
      setDropZoneState('success', result)
    } catch (err) {
      setDropZoneState('error', String(err))
    }
  }
})
