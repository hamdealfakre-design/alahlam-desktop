<script>
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';

  let dashboard = { revenue: 0, expenses: 0, rooms_total: 0, rooms_booked: 0, occupancy_pct: '0%' };
  let bookings = [];
  let rooms = [];
  let ready = false;
  let errorMsg = '';

  async function initApp() {
    try {
      await invoke('init_db');
      const d = await invoke('get_dashboard');
      dashboard = d;
      bookings = await invoke('list_bookings');
      rooms = await invoke('list_rooms');
      ready = true;
    } catch (e) {
      console.error(e);
      errorMsg = String(e);
    }
  }

  onMount(() => {
    initApp();
  });

  async function createBooking() {
    const sample = { guest: 'ضيف تجريبي', room_number: '103', checkin: '2025-05-20', checkout: '2025-05-22', total: 1200 };
    await invoke('create_booking', { payload: sample });
    bookings = await invoke('list_bookings');
    const d = await invoke('get_dashboard');
    dashboard = d;
  }
</script>

<main style="padding:20px;">
  <header style="display:flex;justify-content:space-between;align-items:center;">
    <div style="display:flex;gap:12px;align-items:center">
      <button on:click={() => {}} style="font-size:20px">☰</button>
      <div style="font-weight:800;font-size:1.1rem">ALAHLAM</div>
    </div>
    <div>
      <button style="background:#2d6ef6;color:white;padding:8px 12px;border-radius:8px" on:click={createBooking}>إنشاء حجز تجريبي</button>
    </div>
  </header>

  {#if errorMsg}
    <div style="margin-top:12px;background:#4c1f1f;padding:12px;border-radius:8px;color:#ffd7d7">خطأ: {errorMsg}</div>
  {/if}

  {#if !ready}
    <div style="margin-top:30px">جاري التحميل...</div>
  {:else}
    <section style="display:grid;grid-template-columns:repeat(4,1fr);gap:12px;margin-top:18px">
      <div style="background:rgba(20,39,62,0.9);padding:14px;border-radius:12px">
        <div style="opacity:0.85">الإيرادات</div>
        <div style="font-weight:800;font-size:1.6rem">{dashboard.revenue}</div>
      </div>
      <div style="background:rgba(20,39,62,0.9);padding:14px;border-radius:12px">
        <div style="opacity:0.85">المصروفات</div>
        <div style="font-weight:800;font-size:1.6rem">{dashboard.expenses}</div>
      </div>
      <div style="background:rgba(20,39,62,0.9);padding:14px;border-radius:12px">
        <div style="opacity:0.85">عدد الغرف</div>
        <div style="font-weight:800;font-size:1.6rem">{dashboard.rooms_total}</div>
      </div>
      <div style="background:rgba(20,39,62,0.9);padding:14px;border-radius:12px">
        <div style="opacity:0.85">نسبة الإشغال</div>
        <div style="font-weight:800;font-size:1.6rem">{dashboard.occupancy_pct}</div>
      </div>
    </section>

    <section style="margin-top:20px;display:grid;grid-template-columns:2fr 1fr;gap:12px;">
      <div style="background:rgba(19,39,65,0.96);padding:14px;border-radius:12px">
        <h3 style="margin:0 0 12px">آخر الحجوزات</h3>
        {#if bookings.length === 0}
          <div>لا توجد حجوزات</div>
        {:else}
          <ul>
            {#each bookings as b}
              <li style="padding:8px 0;border-bottom:1px solid rgba(255,255,255,0.04)">{b.guest} — {b.room_number} — {b.checkin} → {b.checkout} — {b.total} ريال</li>
            {/each}
          </ul>
        {/if}
      </div>

      <div style="background:rgba(19,39,65,0.96);padding:14px;border-radius:12px">
        <h3 style="margin:0 0 12px">حالة الغرف</h3>
        <div style="display:grid;grid-template-columns:1fr 1fr;gap:8px">
          {#each rooms as r}
            <div style="padding:8px;border-radius:8px;background:rgba(255,255,255,0.02)">{r.number} — {r.status}</div>
          {/each}
        </div>
      </div>
    </section>
  {/if}
</main>
