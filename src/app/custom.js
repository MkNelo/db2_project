var app = Elm.Main.init({ node: document.getElementById("elm") })

$(document).ready(() => {
    $("#sidebar")
        .mCustomScrollbar({
            theme: "minimal"
        })
    $("#toggler").on("click", () => {
        $("#sidebar").toggleClass("active")
        $("#page-canvas").toggleClass("active")

        $(".collapse.in").toggleClass("in")
        $('a[aria-expanded=true]').attr('aria-expanded', 'false')
    })
})
