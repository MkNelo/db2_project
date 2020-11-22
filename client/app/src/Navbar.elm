module Navbar exposing (navbar, toggler, ofOptions, item)
import Html exposing (..)
import Html.Attributes exposing (..)
import Icons
import Element exposing (el)

type alias NavbarOptions msg =
    { selectedItem : Maybe (NavbarItem msg)
    , options : List (NavbarItem msg)
    }

type alias NavbarItem msg =
    { title : String
    , onCommand : msg
    , icon : Html msg
    }

item: String -> msg -> Html msg -> NavbarItem msg
item = NavbarItem

ofOptions: List (NavbarItem msg) -> NavbarOptions msg
ofOptions els =
    NavbarOptions Nothing els

toggler: String -> List (Html msg) -> Html msg
toggler className childs = 
    button [ class className
           , id "toggler"]
           childs

listItem: NavbarItem msg -> Bool -> Html msg
listItem { title, icon, onCommand } isActive =  
    div [ class <| if isActive then 
                        "border-0 rounded-0 list-group-item list-group-item-action active d-flex flex-row align-items-center" 
                      else 
                        "border-0 rounded-0 list-group-item list-group-item-action d-flex flex-row align-items-center"
        , attribute "data-parent" "#sidebar" 
        , id <| "list-" ++ title
        , attribute "data-toggle" "list" 
        , attribute "role" "tab" ]
        [ icon
        , div [ class "h5 mt-1 ml-3" ] 
                    [ text title ]]

navbar: NavbarOptions msg -> Html msg
navbar opts = 
    nav [ class "active border-right border-dark position-fixed"
        , id "sidebar"
        , attribute "role" "tablist" ]
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
        , div [ class "list-group border-0 text-center text-md-left" ]
              ( opts.options 
              |> List.map ( \elem -> listItem elem <| Maybe.withDefault False
                                                   <| Maybe.map (\pivot -> pivot == elem) 
                                                   <| opts.selectedItem) )]