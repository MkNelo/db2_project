module Navbar exposing (NavbarItem, navbar, toggler)
import Html exposing (..)
import Html.Attributes exposing (..)
import Icons
import Html.Events exposing (onClick)

type alias NavbarItem msg =
    { title : String
    , onCommand : msg
    , icon : Html msg
    }

toggler: String -> List (Html msg) -> Html msg
toggler className childs = 
    button [ class className
           , id "toggler"]
           childs

listItem: NavbarItem msg -> Html msg
listItem { title, icon, onCommand } =
    a [ class "list-group-item list-group-item-action d-flex flex-row align-items-center"
        , attribute "data-parent" "#sidebar" 
        , href <| "#content"
        , onClick onCommand
        , attribute "data-toggle" "list" 
        , attribute "role" "tab"
        , attribute "aria-controls" title ]
        [ icon
        , div [ class "h5 mt-1 ml-3" ] [ text title ]]

navbar: List (NavbarItem msg) -> Html msg
navbar opts = 
    nav [ class "active border-right border-dark position-fixed"
        , id "sidebar" ]
        [ div [ id "upper-o"
              , class "overlay d-flex flex-column bg-transparent align-items-center justify-content-center position-fixed"]
              [ div [ type_ "button"
                    , class "p-2 btn bg-transparent rounded-circle p-0" ] 
                    [ Icons.playCircle "rounded-circle m-0 hover-play" ]
              , div [ class "p-2 h4 text-light" ]
                    [ text "Iniciar Simulación" ]]
        , div [ class "overlay alpha position-fixed" ]
              [ ]
        ,  img [ class "img-thumbnail rounded-0"
               , src "https://www.honeybells.co.uk/assets/images/event-le-mans-24-hour-glamping.jpg"]
               [] 
        , div [ class "list-group list-group-flush"
              , id "sidebar-list"
              , attribute "role" "tablist" ]
              ( opts
              |> List.map listItem )]
