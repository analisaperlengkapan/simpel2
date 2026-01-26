use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Aset {
    pub id: String,
    pub nama: String,
    pub kategori: String,
    pub kode_bmn: String,
    pub merk: Option<String>,
    pub nup: Option<String>,
    pub kondisi: String,
    pub lokasi: String,
    pub nilai_perolehan: Option<f64>,
    pub tanggal_perolehan: Option<String>,
    pub status: String,
    pub keterangan: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PaginatedResponse<T> {
    pub success: bool,
    pub data: Vec<T>,
}

#[cfg(target_arch = "wasm32")]
async fn fetch_asets() -> Result<Vec<Aset>, String> {
    let resp = gloo_net::http::Request::get("/api/pembinaan/perlengkapan/aset?page=1&per_page=100")
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!("API Error: {}", resp.status()));
    }

    let json: PaginatedResponse<Aset> = resp.json().await.map_err(|e| e.to_string())?;
    Ok(json.data)
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_asets() -> Result<Vec<Aset>, String> {
    Ok(vec![])
}

#[component]
pub fn AsetList() -> impl IntoView {
    let asets_resource = LocalResource::new(|| fetch_asets());

    view! {
        <div class="overflow-x-auto">
            <Suspense fallback=move || view! { <p class="text-center py-4">"Memuat data aset..."</p> }>
                {move || match asets_resource.get() {
                    None => view! { <p class="text-center py-4">"Memuat data aset..."</p> }.into_any(),
                    Some(Err(e)) => view! {
                        <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative" role="alert">
                            <strong class="font-bold">"Error! "</strong>
                            <span class="block sm:inline">{e}</span>
                        </div>
                    }.into_any(),
                    Some(Ok(data)) => view! {
                         <table class="min-w-full divide-y divide-gray-200">
                            <thead class="bg-gray-50">
                                <tr>
                                    <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Nama Aset"</th>
                                    <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Kode BMN"</th>
                                    <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Merk/NUP"</th>
                                    <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Kondisi"</th>
                                    <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Lokasi"</th>
                                    <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Status"</th>
                                    <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Aksi"</th>
                                </tr>
                            </thead>
                            <tbody class="bg-white divide-y divide-gray-200">
                                <For
                                    each=move || data.clone()
                                    key=|aset| aset.id.clone()
                                    children=move |aset| {
                                        view! {
                                            <tr>
                                                <td class="px-6 py-4 whitespace-nowrap">
                                                    <div class="text-sm font-medium text-gray-900">{aset.nama}</div>
                                                    <div class="text-sm text-gray-500">{aset.kategori}</div>
                                                </td>
                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{aset.kode_bmn}</td>
                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                                    <div>{aset.merk.unwrap_or_default()}</div>
                                                    <div class="text-xs text-gray-400">"NUP: " {aset.nup.unwrap_or_default()}</div>
                                                </td>
                                                <td class="px-6 py-4 whitespace-nowrap">
                                                    <span class={
                                                        let kondisi_lower = aset.kondisi.to_lowercase();
                                                        move || format!("px-2 inline-flex text-xs leading-5 font-semibold rounded-full {}",
                                                            if kondisi_lower == "baik" { "bg-green-100 text-green-800" }
                                                            else if kondisi_lower.contains("rusak") { "bg-red-100 text-red-800" }
                                                            else { "bg-yellow-100 text-yellow-800" }
                                                        )
                                                    }>
                                                        {aset.kondisi.clone()}
                                                    </span>
                                                </td>
                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{aset.lokasi}</td>
                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{aset.status}</td>
                                                <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                                                    <button class="text-indigo-600 hover:text-indigo-900 mr-2">"Edit"</button>
                                                    <button class="text-red-600 hover:text-red-900">"Hapus"</button>
                                                </td>
                                            </tr>
                                        }
                                    }
                                />
                            </tbody>
                        </table>
                    }.into_any()
                }}
            </Suspense>
        </div>
    }
}
