module Main exposing (Model, Msg, update, view, subscriptions, init)
import Html exposing (..)
import Browser
import Html.Attributes exposing (class)
import Html.Attributes exposing (attribute)
import Navbar exposing (navbar)
import Toolbar exposing (toolbar)
import Html.Attributes exposing (id)
import Html.Attributes exposing (src)
import Icons
import Pages.Welcome
import Navbar exposing (NavbarItem)

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
    , options: List (NavbarItem Msg)
    , selectedReport: Report }

init : () -> (Model, Cmd Msg)
init _ =
    (Model 0
        [ NavbarItem "Inicio" (SelectedPage Home) Icons.home
        , NavbarItem "Ranking" (SelectedPage Ranking) Icons.award
        , NavbarItem "Ranking por Hora" (SelectedPage HourlyRanking) Icons.clock ]
        Home , Cmd.none)

type Report =
    Ranking
    | HourlyRanking
    | Home

type Msg =
    SelectedPage Report

update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
    case msg of
        SelectedPage page -> ( { model | selectedReport = page }, Cmd.none )

subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none

viewReport: Model -> Report -> Html Msg
viewReport { options } report =
    case report of
        _ ->
            Pages.Welcome.page options

view : Model -> Html Msg
view model =
    div [ class "d-flex w-100 h-100" ]
        [ navbar <| model.options
        , div [ id "page-canvas"
              , class "w-100 h-100" ]
              [ toolbar
              , div [ class "page-place" ]
                    [ div [ class "jumbotron container jumbotron-fluid bg-transparent tab-content"
                          , id "#content-pane"]
                          [ div [ class "tab-pane fade"
                                , id "content"]
                                [ viewReport model model.selectedReport ]]]]]
