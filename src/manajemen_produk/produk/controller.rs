use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket_db_pools::Connection;
use rocket::{get, post, put, delete, routes};
use crate::BuildingStoreDB;
use crate::manajemen_produk::produk::model::{Produk, ProdukBuilder, get_produk_factory_registry, get_produk_template_pool};
use crate::manajemen_produk::produk::repository::ProdukRepository;
use crate::manajemen_produk::produk::audit_log_observer::AuditLogObserver;
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ProdukRequest {
    pub nama: String,
    pub kategori: String,
    pub harga: f64,
    pub stok: i32,  // Changed from u32 to i32
    pub deskripsi: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ProdukResponse {
    pub id: Option<i64>,
    pub nama: String,
    pub kategori: String,
    pub harga: f64,
    pub stok: u32,  // Changed from u32 to i32
    pub deskripsi: Option<String>,
}

impl From<Produk> for ProdukResponse {
    fn from(produk: Produk) -> Self {
        Self {
            id: produk.id,
            nama: produk.nama,
            kategori: produk.kategori,
            harga: produk.harga,
            stok: produk.stok,
            deskripsi: produk.deskripsi,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<T>,
}

// Controller functions
#[get("/produk")]
pub async fn list_produk(mut db: Connection<BuildingStoreDB>) -> Json<ApiResponse<Vec<ProdukResponse>>> {
    match ProdukRepository::ambil_semua_produk(&mut *db).await {
        Ok(produk_list) => {
            let response_list = produk_list.into_iter()
                .map(|p| ProdukResponse::from(p))
                .collect();
                
            Json(ApiResponse {
                success: true,
                message: Some("Berhasil mengambil daftar produk".to_string()),
                data: Some(response_list),
            })
        },
        Err(e) => Json(ApiResponse {
            success: false,
            message: Some(format!("Gagal mengambil daftar produk: {}", e)),
            data: None,
        }),
    }
}

#[get("/produk/<id>")]
pub async fn detail_produk(mut db: Connection<BuildingStoreDB>, id: i64) -> Json<ApiResponse<ProdukResponse>> {
    match ProdukRepository::ambil_produk_by_id(&mut *db, id).await {
        Ok(Some(produk)) => Json(ApiResponse {
            success: true,
            message: Some("Berhasil mengambil detail produk".to_string()),
            data: Some(ProdukResponse::from(produk)),
        }),
        Ok(None) => Json(ApiResponse {
            success: false,
            message: Some(format!("Produk dengan ID {} tidak ditemukan", id)),
            data: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            message: Some(format!("Gagal mengambil detail produk: {}", e)),
            data: None,
        }),
    }
}

#[post("/produk", format = "json", data = "<request>")]
pub async fn tambah_produk(
    mut db: Connection<BuildingStoreDB>, 
    request: Json<ProdukRequest>
) -> Json<ApiResponse<ProdukResponse>> {
    // Builder Pattern untuk membuat produk baru
    let produk_result = ProdukBuilder::new(request.nama.clone(), request.kategori.clone())
        .harga(request.harga)
        .stok(request.stok.try_into().unwrap())
        .deskripsi(request.deskripsi.clone().unwrap_or_default())
        .build();
    
    match produk_result {
        Ok(produk) => {
            match ProdukRepository::tambah_produk(&mut *db, &produk).await {
                Ok(id) => {
                    let produk_with_id = Produk::with_id(
                        id, 
                        produk.nama, 
                        produk.kategori, 
                        produk.harga, 
                        produk.stok, 
                        produk.deskripsi
                    );
                    
                    Json(ApiResponse {
                        success: true,
                        message: Some("Berhasil menambahkan produk baru".to_string()),
                        data: Some(ProdukResponse::from(produk_with_id)),
                    })
                },
                Err(e) => Json(ApiResponse {
                    success: false,
                    message: Some(format!("Gagal menyimpan produk: {}", e)),
                    data: None,
                }),
            }
        },
        Err(e) => Json(ApiResponse {
            success: false,
            message: Some(format!("Validasi gagal: {:?}", e)),
            data: None,
        }),
    }
}

#[post("/produk/factory", format = "json", data = "<request>")]
pub async fn tambah_produk_with_factory(
    mut db: Connection<BuildingStoreDB>, 
    request: Json<ProdukRequest>
) -> Json<ApiResponse<ProdukResponse>> {
    // Abstract Factory Pattern - Fixed async lock issue
    let registry = get_produk_factory_registry();
    
    // Create the product before awaiting
    let produk = {
        match registry.lock() {
            Ok(registry) => {
                registry.create_produk(
                    &request.kategori, 
                    request.nama.clone(), 
                    request.harga, 
                    request.stok.try_into().unwrap(), 
                    request.deskripsi.clone()
                )
            },
            Err(_) => {
                return Json(ApiResponse {
                    success: false,
                    message: Some("Gagal mengakses registry factory".to_string()),
                    data: None,
                });
            }
        }
    };
    
    match produk {
        Some(produk) => {
            match ProdukRepository::tambah_produk(&mut *db, &produk).await {
                Ok(id) => {
                    let produk_with_id = Produk::with_id(
                        id, 
                        produk.nama, 
                        produk.kategori, 
                        produk.harga, 
                        produk.stok, 
                        produk.deskripsi
                    );
                    
                    Json(ApiResponse {
                        success: true,
                        message: Some("Berhasil menambahkan produk baru dengan factory".to_string()),
                        data: Some(ProdukResponse::from(produk_with_id)),
                    })
                },
                Err(e) => Json(ApiResponse {
                    success: false,
                    message: Some(format!("Gagal menyimpan produk: {}", e)),
                    data: None,
                }),
            }
        },
        None => Json(ApiResponse {
            success: false,
            message: Some(format!("Factory untuk kategori '{}' tidak ditemukan", request.kategori)),
            data: None,
        }),
    }
}

#[post("/produk/template/<template_key>", format = "json", data = "<request>")]
pub async fn tambah_produk_from_template(
    mut db: Connection<BuildingStoreDB>,
    template_key: String,
    request: Json<ProdukRequest>
) -> Json<ApiResponse<ProdukResponse>> {
    // Object Pool Pattern - Fixed async lock issue
    let template_pool = get_produk_template_pool();
    
    // Get template before awaiting
    let mut produk = {
        match template_pool.lock() {
            Ok(pool) => {
                pool.create_from_template(&template_key, request.harga, request.stok.try_into().unwrap())
            },
            Err(_) => {
                return Json(ApiResponse {
                    success: false,
                    message: Some("Gagal mengakses template pool".to_string()),
                    data: None,
                });
            }
        }
    };
    
    match produk {
        Some(ref mut produk) => {
            // Update nama jika disediakan
            if !request.nama.trim().is_empty() {
                produk.nama = request.nama.clone();
            }
            
            // Update deskripsi jika disediakan
            if request.deskripsi.is_some() {
                produk.deskripsi = request.deskripsi.clone();
            }
            
            match ProdukRepository::tambah_produk(&mut *db, &produk).await {
                Ok(id) => {
                    let produk_with_id = Produk::with_id(
                        id, 
                        produk.nama.clone(), 
                        produk.kategori.clone(), 
                        produk.harga, 
                        produk.stok, 
                        produk.deskripsi.clone()
                    );
                    
                    Json(ApiResponse {
                        success: true,
                        message: Some("Berhasil menambahkan produk dari template".to_string()),
                        data: Some(ProdukResponse::from(produk_with_id)),
                    })
                },
                Err(e) => Json(ApiResponse {
                    success: false,
                    message: Some(format!("Gagal menyimpan produk: {}", e)),
                    data: None,
                }),
            }
        },
        None => Json(ApiResponse {
            success: false,
            message: Some(format!("Template dengan key '{}' tidak ditemukan", template_key)),
            data: None,
        }),
    }
}

#[put("/produk/<id>", format = "json", data = "<request>")]
pub async fn update_produk(
    mut db: Connection<BuildingStoreDB>,
    id: i64,
    request: Json<ProdukRequest>
) -> Json<ApiResponse<ProdukResponse>> {
    // Prototype Pattern - Clone dengan modifikasi
    let produk_result = match ProdukRepository::ambil_produk_by_id(&mut *db, id).await {
        Ok(Some(_)) => {
            // Menggunakan builder untuk memperbarui produk
            let updated_produk = ProdukBuilder::new(request.nama.clone(), request.kategori.clone())
                .id(id)
                .harga(request.harga)
                .stok(request.stok.try_into().unwrap())
                .deskripsi(request.deskripsi.clone().unwrap_or_default())
                .build();
                
            updated_produk
        },
        Ok(None) => {
            return Json(ApiResponse {
                success: false,
                message: Some(format!("Produk dengan ID {} tidak ditemukan", id)),
                data: None,
            })
        },
        Err(e) => {
            return Json(ApiResponse {
                success: false,
                message: Some(format!("Gagal mengambil produk untuk update: {}", e)),
                data: None,
            })
        }
    };
    
    match produk_result {
        Ok(updated_produk) => {
            match ProdukRepository::update_produk(&mut *db, id, &updated_produk).await {
                Ok(true) => {
                    Json(ApiResponse {
                        success: true,
                        message: Some("Berhasil memperbarui produk".to_string()),
                        data: Some(ProdukResponse::from(updated_produk)),
                    })
                },
                Ok(false) => {
                    Json(ApiResponse {
                        success: false,
                        message: Some(format!("Produk dengan ID {} tidak ditemukan", id)),
                        data: None,
                    })
                },
                Err(e) => {
                    Json(ApiResponse {
                        success: false,
                        message: Some(format!("Gagal memperbarui produk: {}", e)),
                        data: None,
                    })
                }
            }
        },
        Err(e) => {
            Json(ApiResponse {
                success: false,
                message: Some(format!("Validasi gagal: {:?}", e)),
                data: None,
            })
        }
    }
}

#[delete("/produk/<id>")]
pub async fn hapus_produk(
    mut db: Connection<BuildingStoreDB>, 
    id: i64
) -> Json<ApiResponse<()>> {
    match ProdukRepository::hapus_produk(&mut *db, id).await {
        Ok(true) => {
            Json(ApiResponse {
                success: true,
                message: Some(format!("Produk dengan ID {} berhasil dihapus", id)),
                data: None,
            })
        },
        Ok(false) => {
            Json(ApiResponse {
                success: false,
                message: Some(format!("Produk dengan ID {} tidak ditemukan", id)),
                data: None,
            })
        },
        Err(e) => {
            Json(ApiResponse {
                success: false,
                message: Some(format!("Gagal menghapus produk: {}", e)),
                data: None,
            })
        }
    }
}

// Implementasi Filter Produk
#[get("/produk/kategori/<kategori>")]
pub async fn filter_produk_by_kategori(
    mut db: Connection<BuildingStoreDB>,
    kategori: String
) -> Json<ApiResponse<Vec<ProdukResponse>>> {
    match ProdukRepository::filter_produk_by_kategori(&mut *db, &kategori).await {
        Ok(produk_list) => {
            let response_list = produk_list.into_iter()
                .map(|p| ProdukResponse::from(p))
                .collect();
                
            Json(ApiResponse {
                success: true,
                message: Some(format!("Berhasil filter produk berdasarkan kategori '{}'", kategori)),
                data: Some(response_list),
            })
        },
        Err(e) => Json(ApiResponse {
            success: false,
            message: Some(format!("Gagal filter produk: {}", e)),
            data: None,
        }),
    }
}

#[get("/produk/harga?<min>&<max>")]
pub async fn filter_produk_by_price(
    mut db: Connection<BuildingStoreDB>,
    min: f64,
    max: f64
) -> Json<ApiResponse<Vec<ProdukResponse>>> {
    match ProdukRepository::filter_produk_by_price_range(&mut *db, min, max).await {
        Ok(produk_list) => {
            let response_list = produk_list.into_iter()
                .map(|p| ProdukResponse::from(p))
                .collect();
                
            Json(ApiResponse {
                success: true,
                message: Some(format!("Berhasil filter produk dengan harga {} - {}", min, max)),
                data: Some(response_list),
            })
        },
        Err(e) => Json(ApiResponse {
            success: false,
            message: Some(format!("Gagal filter produk: {}", e)),
            data: None,
        }),
    }
}

#[get("/produk/stok?<min_stok>")]
pub async fn filter_produk_by_stock(
    mut db: Connection<BuildingStoreDB>,
    min_stok: i32  // Changed from u32 to i32
) -> Json<ApiResponse<Vec<ProdukResponse>>> {
    match ProdukRepository::filter_produk_by_stock_availability(&mut *db, min_stok.try_into().unwrap()).await {
        Ok(produk_list) => {
            let response_list = produk_list.into_iter()
                .map(|p| ProdukResponse::from(p))
                .collect();
                
            Json(ApiResponse {
                success: true,
                message: Some(format!("Berhasil filter produk dengan stok minimal {}", min_stok)),
                data: Some(response_list),
            })
        },
        Err(e) => Json(ApiResponse {
            success: false,
            message: Some(format!("Gagal filter produk: {}", e)),
            data: None,
        }),
    }
}

// Prototype pattern implementation endpoint
#[post("/produk/<id>/clone_with_price", format = "json", data = "<request>")]
pub async fn clone_produk_with_price(
    mut db: Connection<BuildingStoreDB>,
    id: i64,
    request: Json<ProdukRequest>
) -> Json<ApiResponse<ProdukResponse>> {
    match ProdukRepository::ambil_produk_by_id(&mut *db, id).await {
        Ok(Some(existing_produk)) => {
            match existing_produk.clone_with_new_price(request.harga) {
                Ok(cloned_produk) => {
                    // Add custom nama if provided, otherwise keep existing
                    let final_produk = if !request.nama.trim().is_empty() {
                        Produk {
                            id: None,
                            nama: request.nama.clone(),
                            ..cloned_produk
                        }
                    } else {
                        cloned_produk
                    };
                    
                    match ProdukRepository::tambah_produk(&mut *db, &final_produk).await {
                        Ok(new_id) => {
                            let produk_with_id = Produk::with_id(
                                new_id,
                                final_produk.nama,
                                final_produk.kategori,
                                final_produk.harga,
                                final_produk.stok,
                                final_produk.deskripsi
                            );
                            
                            Json(ApiResponse {
                                success: true,
                                message: Some("Berhasil membuat clone produk dengan harga baru".to_string()),
                                data: Some(ProdukResponse::from(produk_with_id)),
                            })
                        },
                        Err(e) => Json(ApiResponse {
                            success: false,
                            message: Some(format!("Gagal menyimpan produk clone: {}", e)),
                            data: None,
                        }),
                    }
                },
                Err(e) => Json(ApiResponse {
                    success: false,
                    message: Some(format!("Validasi gagal: {:?}", e)),
                    data: None,
                }),
            }
        },
        Ok(None) => Json(ApiResponse {
            success: false,
            message: Some(format!("Produk dengan ID {} tidak ditemukan", id)),
            data: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            message: Some(format!("Gagal mengambil produk: {}", e)),
            data: None,
        }),
    }
}

#[post("/produk/<id>/update_stock", format = "json", data = "<new_stok>")]
pub async fn update_stock(
    mut db: Connection<BuildingStoreDB>,
    id: i64,
    new_stok: Json<i32>,
) -> Json<ApiResponse<()>> {
    // Fixed async issue with observers
    match ProdukRepository::ambil_produk_by_id(&mut *db, id).await {
        Ok(Some(mut produk)) => {
            // Create the observer
            let observer = Arc::new(AuditLogObserver::new());
            
            // Add observer to produk
            produk.add_observer(observer);
            
            // Update stok dan trigger observer
            produk.set_stok((*new_stok).try_into().unwrap());
            
            // Simpan ke database
            match ProdukRepository::update_produk(&mut *db, id, &produk).await {
                Ok(_) => Json(ApiResponse {
                    success: true,
                    message: Some("Stok produk berhasil diperbarui".into()),
                    data: None,
                }),
                Err(e) => Json(ApiResponse {
                    success: false,
                    message: Some(format!("Gagal menyimpan perubahan stok: {}", e)),
                    data: None,
                }),
            }
        }
        Ok(None) => Json(ApiResponse {
            success: false,
            message: Some(format!("Produk ID {} tidak ditemukan", id)),
            data: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            message: Some(format!("Gagal mengambil produk: {}", e)),
            data: None,
        }),
    }
}

// Fungsi untuk mendaftarkan semua routes
pub fn routes() -> Vec<rocket::Route> {
    routes![
        list_produk,
        detail_produk,
        tambah_produk,
        tambah_produk_with_factory,
        tambah_produk_from_template,
        update_produk,
        hapus_produk,
        filter_produk_by_kategori,
        filter_produk_by_price,
        filter_produk_by_stock,
        clone_produk_with_price,
        update_stock,
    ]
}