module Main exposing (Model, Msg, update, view, subscriptions, init)
import Html exposing (..)
import Browser
import Html.Attributes exposing (class)
import Html.Attributes exposing (attribute)
import Navbar exposing (navbar)
import Toolbar exposing (toolbar)
import Html.Attributes exposing (id)
import Navbar exposing (ofOptions)
import Navbar exposing (item)
import Html.Attributes exposing (src)
import Icons

main : Program () Model Msg
main =
    Browser.element
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
    }

type alias Model =
    { count: Int
    }

init : () -> (Model, Cmd Msg)
init _ =
    (Model 0, Cmd.none)

type Report =
    Ranking
    | HourlyRanking
type Msg =
    SelectedPage Report
    | Message

update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
    case msg of
        _ ->
            (model, Cmd.none)

subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none

view : Model -> Html Msg
view model =
    div [ class "d-flex w-100 h-100" ]
        [ navbar <| ofOptions <| [ item "Ranking" (SelectedPage Ranking) Icons.award
                                 , item "Ranking por Hora" (SelectedPage HourlyRanking) Icons.clock]
        , div [ id "page-canvas"
              , class "w-100 h-100" ]
              [ toolbar
              , div [ class "bg-transparent page-place" ]
                    [ div [ class "jumbotron container jumbotron-fluid bg-transparent" ]
                          [ div [ class "mx-2 h-100" ]
                                [ div [ class "display-4" ]
                                      [ text "Bienvenido" ]
                                , p [ class "lead mx-2" ] [ text """Esta es una aplicación de reportaje para "Sistemas de bases de datos 2" basada en la carrera de 24h de Le Mans,
                                                              recopilando datos desde 1970 hasta 1979."""]
                                , hr [ class "bg-dark my-2" ] []
                                , div [ class "h2" ]
                                      [ text "Reportes" ]
                                , p [ class "mx-2 " ]
                                    [ text "Se implementaron los primeros 5 reportes de la decena que deben implementarse:" ]
                                , ul []
                                     [ li [] [text "Ranking por hora"]]]]]]]
