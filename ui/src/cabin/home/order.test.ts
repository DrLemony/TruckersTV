import { describe, expect, it } from 'vitest'
import { APPS_SHOWN, loadOrder, moved } from './order'

function storage(value: string | null): Storage {
  return { getItem: () => value } as unknown as Storage
}

describe('the order of the apps', () => {
  it('moves an app to where another is, either way', () => {
    expect(moved(['youtube', 'spotify', 'tv'], 'youtube', 'tv')).toEqual(['spotify', 'tv', 'youtube'])
    expect(moved(['youtube', 'spotify', 'tv'], 'tv', 'youtube')).toEqual(['tv', 'youtube', 'spotify'])
  })

  it('reads back the order kept, leaving out what is gone and adding what is new', () => {
    const order = loadOrder(storage('["chrome","gone","youtube"]'))
    expect(order.slice(0, 2)).toEqual(['chrome', 'youtube'])
    expect([...order].sort()).toEqual([...APPS_SHOWN].sort())
    expect(loadOrder(storage('not json'))).toEqual(APPS_SHOWN)
    expect(loadOrder(storage(null))).toEqual(APPS_SHOWN)
  })
})
