module Pages.Welcome exposing (page)

import Html exposing (..)
import Html.Attributes exposing (attribute, class, href)
import Html.Events exposing (onClick)
import Navbar exposing (NavbarItem)


reportItem : NavbarItem msg -> Html msg
reportItem { title, onCommand } =
    li []
        [ a
            [ href <| "#content"
            , attribute "data-toggle" "list"
            , onClick onCommand
            ]
            [ text title ]
        ]


page : List (NavbarItem msg) -> Html msg
page list =
    div [ class "mx-2 h-100 container-fluid" ]
        [ div [ class "display-4" ]
            [ text "Bienvenido" ]
        , p [ class "lead mx-2" ] [ text """Esta es una aplicación de reportaje para "Sistemas de bases de datos 2" basada en la carrera de 24h de Le Mans,
                                                              recopilando datos desde 1970 hasta 1979.""" ]
        , hr [ class "bg-dark my-2" ] []
        , div [ class "h2" ]
            [ text "Reportes" ]
        , p [ class "mx-2 " ]
            [ text "Se implementaron los primeros 5 reportes de la decena que deben implementarse:" ]
        , ul [ class "mx-4 list-group list-group-flush" ] <|
            List.map reportItem list
        ]
