import { useCallback, useEffect, useRef, useState, type PointerEvent } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { createDragSession } from './dragSession.mjs';

export function useWidgetDrag(enabled: boolean, onError: (message: string) => void) {
  const active = useRef<{ session: ReturnType<typeof createDragSession>; element: HTMLElement; id: number } | null>(null);
  const [dragging, setDragging] = useState(false);
  const finish = useCallback((cancel = false) => {
    const gesture = active.current;
    if (!gesture) return;
    active.current = null;
    if (gesture.element.hasPointerCapture(gesture.id)) gesture.element.releasePointerCapture(gesture.id);
    void gesture.session.finish(cancel).catch(error => onError(String(error))).finally(() => setDragging(false));
  }, [onError]);
  useEffect(() => { if (!enabled) finish(true); }, [enabled, finish]);
  useEffect(() => {
    const cancel = () => finish(true);
    const escape = (event: KeyboardEvent) => { if (event.key === 'Escape') cancel(); };
    window.addEventListener('keydown', escape);
    window.addEventListener('blur', cancel);
    return () => {
      window.removeEventListener('keydown', escape);
      window.removeEventListener('blur', cancel);
      cancel();
    };
  }, [finish]);
  const onPointerDown = useCallback((event: PointerEvent<HTMLElement>) => {
    if (!enabled || active.current || event.button !== 0 || event.pointerType !== 'mouse') return;
    if ((event.target as HTMLElement).closest('button, input, textarea, select, a, [data-no-drag]')) return;
    event.preventDefault();
    event.stopPropagation();
    event.currentTarget.focus({ preventScroll: true });
    event.currentTarget.setPointerCapture(event.pointerId);
    const session = createDragSession(invoke);
    active.current = { session, element: event.currentTarget, id: event.pointerId };
    onError('');
    setDragging(true);
    void session.ready.catch(error => { onError(String(error)); finish(true); });
  }, [enabled, finish, onError]);
  const onPointerMove = useCallback(() => {
    const gesture = active.current;
    if (gesture) void gesture.session.move().catch(error => { onError(String(error)); finish(true); });
  }, [finish, onError]);
  return { dragging, handlers: {
    onPointerDown, onPointerMove,
    onPointerUp: () => finish(),
    onPointerCancel: () => finish(true),
    onLostPointerCapture: () => finish(),
  } };
}
