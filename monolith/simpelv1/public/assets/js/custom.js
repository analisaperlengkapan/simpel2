const allowedFileTypes = [
    ".doc",
    ".docx",
    ".xls",
    ".xlsx",
    ".ppt",
    ".pptx",
    ".pdf",
    "image/jpg",
    "image/jpeg",
    "image/png",
    ".Zip",
    ".rar",
    ".tgz",
    ".7zip",
    ".tar",
];
const dtLayout = "itlpr";
const dtLayoutFilter = "iltpr";
const dtLayoutPrint = "Bltipr";

// $.extend(true, $.fn.dataTable.defaults, {
//     columnDefs: [{ className: "dt-center", targets: "_all" }],
// });
(function ($) {
    $.ajaxSetup({
        headers: {
            "X-CSRF-TOKEN": $('meta[name="csrf-token"]').attr("content"),
        },
    });

    $.fn.serializeFormJSON = function () {
        var o = {};
        var a = this.serializeArray();
        $.each(a, function () {
            if (o[this.name]) {
                if (!o[this.name].push) {
                    o[this.name] = [o[this.name]];
                }
                o[this.name].push(this.value || "");
            } else {
                o[this.name] = this.value || "";
            }
        });
        return o;
    };
})(jQuery);

$(document)
    .on("ajaxStart", () => $.LoadingOverlay("show"))
    .on("ajaxStop", () => $.LoadingOverlay("hide"));

$(function () {
    $("#backButton").on("click", function () {
        goBack();
    });

    $(".dayjs").each((i, elm) => {
        const tgl = $(elm).text();
        const momentnya = moment(tgl);
        $(elm).text(momentnya.isValid() ? momentnya.fromNow() : "-");
    });

    const ajaxFormOptions = {
        beforeSubmit: showRequest, // pre-submit callback
        success: showResponse, // post-submit callback
        error: showError, // post-submit callback
        dataType: "json",
        //forceSync: true,
        // other available options:
        //dataType:  null        // 'xml', 'script', or 'json' (expected server response type)
        // $.ajax options can be used here too, for example:
        //timeout:   3000
    };

    // bind form using 'ajaxForm'
    $(".ajaxForm").ajaxForm(ajaxFormOptions);

    $(document)
        .on("click", ".deleteRowDT", function () {
            const title = $(this).data("title");
            const target = $(this).data("target");
            const tableId = $(this).data("table");

            swal(`Yakin akan menghapus ${title}?`, {
                icon: "info",
                dangerMode: true,
                buttons: true,
            }).then((res) => {
                if (res && target !== "#") {
                    $.ajax({
                        url: target,
                        method: "DELETE",
                        contentType: "application/json",
                        // beforeSend: () => $.LoadingOverlay("show"),
                        success: (res) => {
                            notify({
                                type: res.status || "success",
                                message: res.msg || "Berhasil",
                            });
                            const table = $("#" + tableId).DataTable();
                            table.ajax.reload();
                        },
                        error: (xhr, status, error) => {
                            console.log(xhr);
                            console.log(status);
                            console.log(error);
                        },
                        // complete: () => $.LoadingOverlay("hide"),
                        // complete: (a, b) => {
                        //     console.log(a);
                        //     console.log(b);
                        //     $.LoadingOverlay("hide");
                        // },
                    });
                    // .on("done", (res) => {
                    //     notify({
                    //         type: res.status || "success",
                    //         message: res.msg || "Berhasil",
                    //     });
                    //     const table = $(tableId).DataTable();
                    //     table.ajax.reload();
                    // })
                    // .on("fail", (a, b, c) => {
                    //     console.log(a);
                    //     console.log(b);
                    // })
                    // .always(() => $.LoadingOverlay("hide"));
                }
            });
        })
        .on("onfocusout change", ".column-filter", function () {
            const dt = $("#" + tableId).DataTable();
            dt.column($(this).data("column")).search(this.value).draw();
        })
        .on("keypress", ".column-filter", function (e) {
            var key = e.which;
            if (key == 13) {
                const dt = $("#" + tableId).DataTable();
                dt.column($(this).data("column")).search(this.value).draw();
            }
        });
});

function showError(response, b, c) {
    notify({
        type: "warning",
        message: response?.responseJSON?.message || "Data Tidak Lengkap!",
    });
    // $.LoadingOverlay("hide");
}
function showRequest(formData, jqForm, options) {
    $("form .input-mask").each((i, elm) => {
        const id = $(elm).attr("id");
        const newVal = $(elm).cleanVal();
        formData.push({ name: id, value: newVal });
    });
    // $('form .tinymce').each((i, elm) =>
    //   formData.push({ name: $(this).attr('name'), value: tinymce.getContent() })
    // );
    // $.LoadingOverlay("show");

    var $this = $('button[type="submit"]');
    $this.button("disabled");

    // $('#myLoader').modal('show');

    // here we could return false to prevent the form from being submitted;
    // returning anything other than false will allow the form submit to continue
    return true;
}

function showResponse(responseText, statusText, xhr, form) {
    // console.log(responseText);
    // console.log(statusText);
    // console.log(xhr);
    // $.LoadingOverlay("hide");
    if (responseText.status == "error") {
        swal("Gagal!", responseText.message || "Terjadi Kesalahan!", "error");
    } else if (responseText.status == "warning") {
        notify({
            type: "warning",
            title: "Warning",
            message: mapValidationError(responseText.message),
            delay: 7000,
        });
    } else {
        swal("Sukses!", responseText.message || "Ok", "success", {
            timer: 2000,
            button: false,
        }).then(() =>
            responseText.callback
                ? callbackResponse(responseText.callback)
                : goBack()
        );
    }
}

function mapValidationError(error) {
    if (typeof error === "object") {
        let message = "";
        for (const [field, msg] of Object.entries(error)) {
            message += `${msg} <br/>`;
        }
        return message;
    } else {
        return error;
    }
}

function callbackResponse(payload) {
    switch (payload.type) {
        case "redirect":
            window.location.href = payload.url;
            break;
        case "uppyUpload":
            uppyUpload(payload.body || {});
            break;
        default:
            break;
    }
}

function goBack() {
    const currentUrl = window.location.href;
    let part = currentUrl.split("/");
    if (part.length == 7 && part[3] == "asset") {
        part.pop();
    }
    part.pop();
    const newUrl = part.join("/");
    window.location.href = newUrl;
}
function int2money(money, comma = false) {
    /*
          if(!isInt(money)){

              money = money.replace(".","");

              money = money.replace(",",".");

          }

          */

    money = parseFloat(money);

    //alert("0 = "+money);

    money = numeral(money).format(`0,0${comma ? ".00" : ""}`);

    //alert("1 = "+money);

    money = money.replace(".", "_");

    //alert("2 = "+money);

    money = money.replace(/,/g, ".");

    //alert("3 = "+money);

    money = money.replace("_", ",");

    //alert("4 = "+money);

    return money;
}

function money2int(money) {
    if (typeof money !== "undefined") {
        money = money.replace(/\./g, "");

        //alert("4 = "+money);

        money = money.replace(",", ".");

        //alert("5 = "+money);

        money = parseInt(money);

        //alert("6 = "+money);
    } else {
        money = 0;
    }

    return !isNaN(money) ? money : 0;
}

function getShortMonth(month = null) {
    const months = [
        {
            id: 1,
            textId: "01",
            text: "Januari",
            shortText: "Jan",
        },
        {
            id: 2,
            textId: "02",
            text: "Februari",
            shortText: "Feb",
        },
        {
            id: 3,
            textId: "03",
            shortText: "Mar",
            text: "Maret",
        },
        {
            id: 4,
            textId: "04",
            shortText: "Apr",
            text: "April",
        },
        {
            id: 5,
            textId: "05",
            shortText: "Mei",
            text: "Mei",
        },
        {
            id: 6,
            textId: "06",
            shortText: "Jun",
            text: "Juni",
        },
        {
            id: 7,
            textId: "07",
            shortText: "Jul",
            text: "Juli",
        },
        {
            id: 8,
            textId: "08",
            shortText: "Aug",
            text: "Agustus",
        },
        {
            id: 9,
            textId: "09",
            shortText: "Sep",
            text: "September",
        },
        {
            id: 10,
            textId: "10",
            shortText: "Okt",
            text: "Oktober",
        },
        {
            id: 11,
            textId: "11",
            shortText: "Nov",
            text: "November",
        },
        {
            id: 12,
            textId: "12",
            shortText: "Des",
            text: "Desember",
        },
    ];
    if (!month) return months;
    return months.find((isi) => isi.id == month);
}

function dateFormat(completeDate, ret = "all") {
    const [date, time] = completeDate.split(" ");
    const exDate = date.split("-");
    if (ret === "all")
        return `${exDate[2]}-${exDate[1]}-${exDate[0]} ${time || ""}`;
    if (ret === "m") return exDate[1];
    if (ret === "y") return exDate[0];
    return date;
}

function dateFormatIndo(completeDate, short = false) {
    if (!completeDate) return "";
    const [date, time] = completeDate.split(" ");
    const exDate = date.split("-");
    const month = short
        ? getShortMonth(exDate[1]).shortText
        : getShortMonth(exDate[1]).text;
    return `${exDate[2]}-${month}-${exDate[0]} ${time?.substr(0, 8) || ""}`;
}

function toPrimitiveArray(arr, key) {
    return arr.map((data) => data[key]);
}

function iconStatus(status) {
    switch (status) {
        case "success":
            return "fas fa-check";
        case "info":
            return "fas fa-info";
        case "warning":
            return "fas fa-exclamation-triangle";
        case "danger":
            return "fas fa-alert-triangle";
        default:
            return "fas fa-envelope-open";
    }
}

function notify(params) {
    const { type, title, message, url, icon, delay, cb } = params;

    Toastify({
        text: message,
        duration: delay,
        // destination: "https://github.com/apvarun/toastify-js",
        // newWindow: true,
        close: true,
        gravity: "top", // `top` or `bottom`
        position: "right", // `left`, `center` or `right`
        stopOnFocus: true, // Prevents dismissing of toast on hover
        className: `bg-${type}`,
        onClick: () => cb || {}, // Callback after click
    }).showToast();
}

function formatSelect2Data({ array, text, value }) {
    return array.map((arr) => ({ id: arr[value], text: arr[text] }));
}

function shortId(length = 8) {
    var chars =
        "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    var id = "";

    for (var i = 0; i < length; i++) {
        var randomIndex = Math.floor(Math.random() * chars.length);
        id += chars.charAt(randomIndex);
    }

    return id;
}

function buttonPrintDt(title, url) {
    console.log('buttonPrintDt called with:', {title, url});
    return [
        {
            text: "Excel Semua Kolom",
            extend: "excel",
            action: function (e, dt, node, config) {
                console.log('Excel Semua Kolom clicked');
                let param = dt.ajax.params();
                let visible = $("#setting-column").val().toString();
                console.log('Params:', param);
                console.log('Visible:', visible);
                console.log('URL:', url + "/cetakExcel?isKolom=all&&visible=" + visible);
                
                // Show loading indicator
                $.LoadingOverlay("show");
                
                $.ajax({
                    url: url + "/cetakExcel?isKolom=all&&visible=" + visible,
                    type: "POST",
                    data: param,
                    timeout: 1800000, // 30 menit timeout untuk server
                    xhrFields: {
                        responseType: "blob",
                    },
                    success: function (result, status, xhr) {
                        console.log('Excel success:', status);
                        $.LoadingOverlay("hide");
                        
                        var disposition = xhr.getResponseHeader("content-disposition");
                        var matches = /"([^"]*)"/.exec(disposition);
                        var filename = matches != null && matches[1] ? matches[1] : title + ".xlsx";
                        
                        var blob = new Blob([result], {
                            type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                        });
                        var link = document.createElement("a");
                        link.href = window.URL.createObjectURL(blob);
                        link.download = filename;
                        document.body.appendChild(link);
                        link.click();
                        document.body.removeChild(link);
                    },
                    error: function (xhr, status, error) {
                        console.log('Excel error:', {xhr, status, error});
                        $.LoadingOverlay("hide");
                        
                        // Try to parse error response
                        if (xhr.responseType === 'blob') {
                            var reader = new FileReader();
                            reader.onload = function() {
                                try {
                                    var errorResponse = JSON.parse(reader.result);
                                    alert('Error: ' + (errorResponse.error || 'Unknown error'));
                                } catch(e) {
                                    alert('Error downloading Excel file');
                                }
                            };
                            reader.readAsText(xhr.response);
                        } else {
                            alert('Error downloading Excel file');
                        }
                    },
                });
            },
        },
        {
            text: "Excel",
            extend: "excel",
            action: function (e, dt, node, config) {
                console.log('Excel clicked');
                let param = dt.ajax.params();
                let visible = $("#setting-column").val().toString();
                console.log('Params:', param);
                console.log('Visible:', visible);
                console.log('URL:', url + "/cetakExcel?isKolom=part&&visible=" + visible);
                
                // Show loading indicator
                $.LoadingOverlay("show");
                
                $.ajax({
                    url: url + "/cetakExcel?isKolom=part&&visible=" + visible,
                    type: "POST",
                    data: param,
                    timeout: 1800000, // 30 menit timeout untuk server
                    xhrFields: {
                        responseType: "blob",
                    },
                    success: function (result, status, xhr) {
                        console.log('Excel success:', status);
                        $.LoadingOverlay("hide");
                        
                        var disposition = xhr.getResponseHeader("content-disposition");
                        var matches = /"([^"]*)"/.exec(disposition);
                        var filename = matches != null && matches[1] ? matches[1] : title + ".xlsx";
                        
                        var blob = new Blob([result], {
                            type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                        });
                        var link = document.createElement("a");
                        link.href = window.URL.createObjectURL(blob);
                        link.download = filename;
                        document.body.appendChild(link);
                        link.click();
                        document.body.removeChild(link);
                    },
                    error: function (xhr, status, error) {
                        console.log('Excel error:', {xhr, status, error});
                        $.LoadingOverlay("hide");
                        
                        // Try to parse error response
                        if (xhr.responseType === 'blob') {
                            var reader = new FileReader();
                            reader.onload = function() {
                                try {
                                    var errorResponse = JSON.parse(reader.result);
                                    alert('Error: ' + (errorResponse.error || 'Unknown error'));
                                } catch(e) {
                                    alert('Error downloading Excel file');
                                }
                            };
                            reader.readAsText(xhr.response);
                        } else {
                            alert('Error downloading Excel file');
                        }
                    },
                });
            },
        },
        {
            extend: "pdf",
            action: function (e, dt, node, config) {
                console.log('PDF clicked');
                let param = dt.ajax.params();
                console.log('Params:', param);
                console.log('URL:', url + "/cetakPdf");
                
                // Show loading indicator
                $.LoadingOverlay("show");
                
                $.ajax({
                    url: url + "/cetakPdf",
                    type: "POST",
                    data: param,
                    timeout: 900000, // 15 menit timeout untuk PDF
                    xhrFields: {
                        responseType: "blob",
                    },
                    success: function (result, status, xhr) {
                        console.log('PDF success:', status);
                        $.LoadingOverlay("hide");
                        
                        var disposition = xhr.getResponseHeader("content-disposition");
                        var matches = /"([^"]*)"/.exec(disposition);
                        var filename = matches != null && matches[1] ? matches[1] : title + ".pdf";
                        
                        var blob = new Blob([result], {
                            type: "application/pdf",
                        });
                        var link = document.createElement("a");
                        link.href = window.URL.createObjectURL(blob);
                        link.download = filename;
                        document.body.appendChild(link);
                        link.click();
                        document.body.removeChild(link);
                    },
                    error: function (xhr, status, error) {
                        console.log('PDF error:', {xhr, status, error});
                        $.LoadingOverlay("hide");
                        
                        // Try to parse error response
                        if (xhr.responseType === 'blob') {
                            var reader = new FileReader();
                            reader.onload = function() {
                                try {
                                    var errorResponse = JSON.parse(reader.result);
                                    alert('Error: ' + (errorResponse.error || 'Unknown error'));
                                } catch(e) {
                                    alert('Error downloading PDF file');
                                }
                            };
                            reader.readAsText(xhr.response);
                        } else {
                            alert('Error downloading PDF file');
                        }
                    },
                });
            },
        },
    ];
}

function int2money(duit) {
    return numeral(duit).format("0,0");
}
function money2int(duit) {
    return numeral(duit).format("0,0");
}
